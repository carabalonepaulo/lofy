use proc_macro2::{Ident, Literal, TokenStream, TokenTree};
use quote::quote;
use venial::{FnParam, Function, Impl, ImplMember};

#[allow(unused, dead_code)]
enum ParamsInfo {
    RawMethod,
    RawMethodMut,

    RawStatic,
    RawStaticMut,

    Method,
    MethodMut,

    Static,
}

fn params_info(func: &Function) -> Option<ParamsInfo> {
    // raw static | instance with no args
    if func.params.len() == 1 {
        match &func.params[0].0 {
            FnParam::Receiver(_) => None,
            FnParam::Typed(_) => {
                todo!()
            }
        }
    // raw method | method with single arg
    } else if func.params.len() == 2 {
        match &func.params[0].0 {
            FnParam::Receiver(param) => {
                // no ref allowed?
                // if param.tk_ref.is_some() {
                //     return None;
                // }

                // only Self allowed, anything else ignored
                if &param.tk_self.to_string() != "self" {
                    return None;
                }

                Some(if param.tk_mut.is_some() {
                    ParamsInfo::RawMethodMut
                } else {
                    ParamsInfo::RawMethod
                })
            }
            FnParam::Typed(_) => None,
        }
    // method | static
    } else {
        None
    }
}

fn gen_raw_method(ty_ident: &Ident, fn_ident: &Ident, fn_str: Literal) -> TokenStream {
    quote! {
        sys::luaL_Reg {
            name: cstr!(#fn_str),
            func: {
                unsafe extern "C" fn step(raw_state: *mut sys::lua_State) -> std::ffi::c_int {
                    let mut state = State::from_raw(raw_state);
                    // TODO: check userdata
                    let mut user_data = unsafe { sys::lua_touserdata(raw_state, 1) as *mut #ty_ident };

                    let self_ref = &mut *user_data;
                    let n = #ty_ident::#fn_ident(self_ref, &mut state);

                    n as std::ffi::c_int
                }
                Some(step)
            },
        }
    }
}

fn gen_raw_mut_method(ty_ident: &Ident, fn_ident: &Ident, fn_str: Literal) -> TokenStream {
    quote! {
        sys::luaL_Reg {
            name: cstr!(#fn_str),
            func: {
                unsafe extern "C" fn step(raw_state: *mut sys::lua_State) -> std::ffi::c_int {
                    let mut state = State::from_raw(raw_state);
                    // TODO: check userdata
                    let mut user_data = unsafe { sys::lua_touserdata(raw_state, 1) as *mut #ty_ident };

                    let self_mut_ref = &mut *user_data;
                    let n = #ty_ident::#fn_ident(self_mut_ref, &mut state);

                    n as std::ffi::c_int
                }
                Some(step)
            },
        }
    }
}

pub fn gen_user_data_impl(ty: Impl) -> TokenStream {
    let ty_ident = {
        if let TokenTree::Ident(ident) = ty.self_ty.tokens[0].clone() {
            ident
        } else {
            unreachable!()
        }
    };

    let mut decls: Vec<TokenStream> = vec![];
    for item in ty.body_items.iter() {
        if let ImplMember::Method(func) = item {
            // only collect public methods
            if func.vis_marker.is_none() {
                continue;
            }

            // no async method allowed
            if func.qualifiers.tk_async.is_some() {
                panic!("async functions not allowed.")
            }

            let fn_ident = &func.name;
            let fn_str = proc_macro2::Literal::string(&fn_ident.to_string());

            // raw methods:
            // - (&self, &mut State)
            // - (&mut self, &mut State)
            //
            // raw static:
            // - (&mut State)
            // - (&mut State)
            //
            // methods:
            // - (&self, ...)
            // - (&mut self, ...)
            //
            // static:
            // - (...)
            // - (...)

            if let Some(info) = params_info(&func) {
                match info {
                    ParamsInfo::RawMethod => {
                        decls.push(gen_raw_method(&ty_ident, fn_ident, fn_str))
                    }
                    ParamsInfo::RawMethodMut => {
                        decls.push(gen_raw_mut_method(&ty_ident, fn_ident, fn_str))
                    }
                    ParamsInfo::RawStatic => todo!(),
                    ParamsInfo::RawStaticMut => todo!(),
                    ParamsInfo::Method => todo!(),
                    ParamsInfo::MethodMut => todo!(),
                    ParamsInfo::Static => todo!(),
                }
            }

            // result type validation is done later by casting to UserData trait.
        }
    }

    let ty_str = proc_macro2::Literal::string(&ty_ident.to_string());

    quote! {
        impl UserData for #ty_ident {
            fn name() -> *const i8 { cstr!(#ty_str) }
            fn functions() -> Vec<sys::luaL_Reg> {
                vec![
                    #(#decls),*
                ]
            }
        }
    }
}
