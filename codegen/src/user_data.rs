use proc_macro2::{Ident, Literal, Punct, Span, TokenStream, TokenTree};
use quote::quote;
use venial::{FnParam, Function, Impl, ImplMember, Punctuated};

#[allow(unused, dead_code)]
enum ParamsInfo {
    RawMethod(bool),

    RawStatic,

    MethodNoArgNoReturn(bool),

    Method(bool, TokenStream, TokenStream),

    Static(TokenStream, TokenStream),
}

fn get_args_ty(func: &Function) -> (TokenStream, TokenStream) {
    let mut params_ty = vec![];
    for (param, _) in &func.params[1..] {
        let FnParam::Typed(param) = param else { continue; };
        params_ty.push(param.ty.clone());
    }
    let return_ty = func.return_ty.clone();
    (quote!((#(#params_ty,)*)), quote!(#return_ty))
}

fn is_self_param(param: &FnParam) -> Option<bool> {
    let FnParam::Receiver(first_param) = param else { return None; };
    if first_param.tk_ref.is_none() || &first_param.tk_self.to_string().to_lowercase() != "self" {
        None
    } else {
        Some(first_param.tk_mut.is_some())
    }
}

fn is_state_param(tokens: &Vec<TokenTree>) -> bool {
    let TokenTree::Punct(punct) = &tokens[0] else { return false; };

    if punct.as_char() != '&' {
        return false;
    }

    let TokenTree::Ident(ident) = &tokens[1] else { return false; };
    ident.to_string().rfind("State").is_some()
}

fn params_info(func: &Function) -> Option<ParamsInfo> {
    // raw static | instance with no args
    if func.params.len() == 1 {
        match &func.params[0].0 {
            FnParam::Receiver(param) => {
                Some(ParamsInfo::MethodNoArgNoReturn(param.tk_mut.is_some()))
            }
            FnParam::Typed(param) => {
                if is_state_param(&param.ty.tokens) {
                    Some(ParamsInfo::RawStatic)
                } else {
                    None
                }
            }
        }
    // raw method | method with single arg
    } else if func.params.len() == 2 {
        let Some(is_mut) = is_self_param(&func.params[0].0) else { return None; };
        let FnParam::Typed(second_param) = &func.params[1].0 else { return None; };
        if is_state_param(&second_param.ty.tokens) {
            Some(ParamsInfo::RawMethod(is_mut))
        } else {
            None
        }
    // method | static => not yet
    } else {
        if let Some(is_mut) = is_self_param(&func.params[0].0) {
            let (args_ty, return_ty) = get_args_ty(&func);
            Some(ParamsInfo::Method(is_mut, args_ty, return_ty))
        } else {
            let (args_ty, return_ty) = get_args_ty(&func);
            Some(ParamsInfo::Static(args_ty, return_ty))
        }
    }
}

// fn gen_raw_method(
//     _is_mut: bool,
//     ty_ident: &Ident,
//     fn_ident: &Ident,
//     fn_str: Literal,
// ) -> TokenStream {
//     quote! {
//         sys::luaL_Reg {
//             name: cstr!(#fn_str),
//             func: {
//                 unsafe extern "C" fn step(raw_state: *mut sys::lua_State) -> std::ffi::c_int {
//                     let mut state = State::from_raw(raw_state);
//                     // TODO: check userdata
//                     let mut user_data = unsafe { sys::lua_touserdata(raw_state, 1) as *mut #ty_ident };
//
//                     let self_ref = &mut *user_data;
//                     let n = #ty_ident::#fn_ident(self_ref, &state);
//
//                     n as std::ffi::c_int
//                 }
//                 Some(step)
//             },
//         }
//     }
// }
fn get_self_ty(is_mut: bool, ty: &Ident) -> TokenStream {
    if is_mut {
        quote!(&mut #ty)
    } else {
        quote!(&#ty)
    }
}

fn gen_raw_method(
    is_mut: bool,
    ty_ident: &Ident,
    fn_ident: &Ident,
    fn_str: Literal,
) -> TokenStream {
    let ty_self = get_self_ty(is_mut, ty_ident);

    quote! {
        sys::luaL_Reg {
            name: cstr!(#fn_str),
            func: {
                unsafe extern "C" fn step(raw_state: *mut sys::lua_State) -> std::ffi::c_int {
                    let mut state = State::from_raw(raw_state);
                    let self_mut_ref = state.cast_to::<#ty_self>(1).unwrap();
                    let n = #ty_ident::#fn_ident(self_mut_ref, &state);
                    n as std::ffi::c_int
                }
                Some(step)
            },
        }
    }
}

// (state: &State)
fn gen_raw_static(ty_ident: &Ident, fn_ident: &Ident, fn_str: Literal) -> TokenStream {
    quote! {
        luajit2_sys::luaL_Reg {
            name: cstr!(#fn_str),
            func: {
                extern "C" fn step(ptr: *mut sys::lua_State) -> std::ffi::c_int {
                    let state = State::from_raw(ptr);
                    <#ty_ident>::#fn_ident(&state) as std::ffi::c_int
                }
                Some(step)
            },
        }
    }
}

fn gen_static(
    ident: &Ident,
    fn_ident: &Ident,
    fn_str: Literal,
    args_ty: TokenStream,
    return_ty: TokenStream,
) -> TokenStream {
    quote! {
        luajit2_sys::luaL_Reg {
            name: cstr!(#fn_str),
            func: {
                extern "C" fn step(ptr: *mut sys::lua_State) -> std::ffi::c_int {
                    let state = State::from_raw(ptr);
                    let len = <#args_ty as FromLua>::len();
                    let idx = len * -1;

                    let args = state.cast_to::<#args_ty>(idx).unwrap();
                    let result = <#ident>::#fn_ident(args);

                    state.push(result);
                    <#return_ty as ToLua>::len() as std::ffi::c_int
                }
                Some(step)
            },
        }
    }
}

fn gen_method(
    is_mut: bool,
    ident: &Ident,
    fn_ident: &Ident,
    fn_str: Literal,
    args_ty: TokenStream,
    return_ty: TokenStream,
) -> TokenStream {
    let self_ty = get_self_ty(is_mut, ident);
    quote! {
        luajit2_sys::luaL_Reg {
            name: cstr!(#fn_str),
            func: {
                extern "C" fn step(ptr: *mut sys::lua_State) -> std::ffi::c_int {
                    let state = State::from_raw(ptr);
                    let len = <#args_ty as FromLua>::len() + 1;
                    let idx = len * -1;

                    let ud = state.cast_to::<#self_ty>(idx).unwrap();
                    let args = state.cast_to::<#args_ty>(idx + 1).unwrap();
                    let result = ud.#fn_ident(args);

                    state.push(result);
                    <#return_ty as ToLua>::len() as std::ffi::c_int
                }
                Some(step)
            },
        }
    }
}

fn gen_method_no_arg_no_return(
    is_mut: bool,
    ud_ty: &Ident,
    fn_ident: &Ident,
    fn_str: Literal,
) -> TokenStream {
    let self_ty = get_self_ty(is_mut, ud_ty);
    quote! {
        luajit2_sys::luaL_Reg {
            name: cstr!(#fn_str),
            func: {
                extern "C" fn step(ptr: *mut sys::lua_State) -> std::ffi::c_int {
                    let state = State::from_raw(ptr);
                    let ud = state.cast_to::<#self_ty>(-1).unwrap();
                    ud.#fn_ident();
                    0
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
            // - (&self, &State)
            // - (&mut self, &State)
            //
            // raw static:
            // - (&State)
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
                    ParamsInfo::RawMethod(is_mut) => {
                        decls.push(gen_raw_method(is_mut, &ty_ident, fn_ident, fn_str))
                    }
                    ParamsInfo::RawStatic => {
                        decls.push(gen_raw_static(&ty_ident, fn_ident, fn_str))
                    }
                    ParamsInfo::Static(args_ty, return_ty) => {
                        decls.push(gen_static(&ty_ident, fn_ident, fn_str, args_ty, return_ty))
                    }
                    ParamsInfo::Method(is_mut, args_ty, return_ty) => decls.push(gen_method(
                        is_mut, &ty_ident, fn_ident, fn_str, args_ty, return_ty,
                    )),
                    ParamsInfo::MethodNoArgNoReturn(is_mut) => decls.push(
                        gen_method_no_arg_no_return(is_mut, &ty_ident, fn_ident, fn_str),
                    ),
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
