use proc_macro;
use proc_macro2::{Ident, Literal, Span, TokenStream, TokenTree};
use quote::quote;
use syn::LitInt;
use venial::{parse_declaration, Declaration, FnParam, Function, Impl, ImplMember};

#[proc_macro]
pub fn generate_to_lua_tuple_impl(attr: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let max_lit = syn::parse_macro_input!(attr as LitInt);
    let max = max_lit.base10_parse::<usize>().unwrap() + 1;
    let mut parts = vec![];
    let alphabet: Vec<char> = (b'A'..b'Z').map(|c| c as char).collect();

    (2..max).for_each(|n| {
        let mut state_push = vec![];
        let mut letters_a = vec![];
        let mut letters_b = vec![];
        let mut where_ch = vec![];
        let len = proc_macro2::Literal::i32_unsuffixed(n as i32);

        (0..n).for_each(|i| {
            let letter = alphabet[i];
            let index = syn::Index::from(i);
            let ch = Ident::new(&letter.to_string(), Span::call_site());

            state_push.push(quote!(state.push(self.#index);));
            letters_a.push(ch.clone());
            letters_b.push(ch.clone());
            where_ch.push(quote!(#ch: ToLua))
        });

        /*
        impl<A, B, C, D, E> ToLua for (A, B, C, D, E)
        where
            A: ToLua,
            B: ToLua,
            C: ToLua,
            D: ToLua,
            E: ToLua,
        {
            fn to_lua(self, state: *mut sys::lua_State) {
                todo!()
            }
        }
        */
        parts.push(quote! {
            impl<#(#letters_a,)*> ToLua for (#(#letters_b,)*)
            where
                #(#where_ch,)*
            {
                fn to_lua(self, state: *mut sys::lua_State) {
                    let mut state = State::from_raw(state);
                    #(#state_push)*
                }

                fn len() -> i32 { #len }
            }
        });
    });

    proc_macro::TokenStream::from(quote!(#(#parts)*))
}

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

fn gen_user_data_impl(ty: Impl) -> TokenStream {
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

#[proc_macro_attribute]
pub fn user_data(
    _attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let token_stream = proc_macro2::TokenStream::from(item);
    let ty = match parse_declaration(token_stream.clone()) {
        Ok(Declaration::Impl(ty)) => ty,
        _ => panic!("user_data attribute can only be used with impl."),
    };

    let user_data_impl = gen_user_data_impl(ty);

    proc_macro::TokenStream::from(quote! {
        #token_stream
        #user_data_impl
    })
}
