use proc_macro2::{Ident, Literal, Punct, Span, TokenStream, TokenTree};
use quote::quote;
use venial::{FnParam, Function, Impl, ImplMember, Punctuated};

//
// Kinds:
// - Raw
//   - Static
//   - Method
// - Static
// - Method
//
// Special:
// - NoArgNoRet
// - NoArgRet
// - ArgNoRet
// - ArgRet
//
struct MethodInfo(
    Option<bool>,
    Option<(Option<TokenStream>, Option<TokenStream>)>,
);

struct MethodMeta {
    owner_ty: Ident,
    method_ty: Ident,
    method_str: Literal,
}

impl MethodInfo {
    fn is_mut(&self) -> bool {
        match self.0 {
            Some(is_mut) => is_mut,
            None => false,
        }
    }

    fn is_raw(&self) -> bool {
        self.1.is_none()
    }

    fn is_method(&self) -> bool {
        self.0.is_some()
    }

    fn has_args(&self) -> bool {
        self.1
            .clone()
            .and_then(|(args, _)| Some(args.is_some()))
            .is_some()
    }

    fn has_return_value(&self) -> bool {
        self.1
            .clone()
            .and_then(|(_, result)| Some(result.is_some()))
            .is_some()
    }
}

#[allow(unused, dead_code)]
enum ParamsInfo {
    Method(MethodInfo), // RawMethod(bool),
                        // RawStatic,

                        // MethodNoArgNoReturn(bool),
                        // Method(bool, TokenStream, TokenStream),

                        // Static(TokenStream, TokenStream),
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

fn params_info(func: &Function) -> Option<MethodInfo> {
    // raw static | instance with no args
    if func.params.len() == 1 {
        match &func.params[0].0 {
            // (&self, a: A) -> B
            FnParam::Receiver(param) => {
                let args = None;
                let result = {
                    if func.return_ty.is_some() {
                        let return_ty = func.return_ty.clone().unwrap();
                        Some(quote!(#return_ty))
                    } else {
                        None
                    }
                };
                Some(MethodInfo(
                    Some(param.tk_mut.is_some()),
                    Some((args, result)),
                ))
            }
            // (state: &State) -> i32
            FnParam::Typed(param) => {
                if is_state_param(&param.ty.tokens) {
                    Some(MethodInfo(None, None))
                } else {
                    None
                }
            }
        }
    // raw method | method with single arg | static
    } else if func.params.len() == 2 {
        let Some(is_mut) = is_self_param(&func.params[0].0) else { return None; };
        let FnParam::Typed(second_param) = &func.params[1].0 else { return None; };
        if is_state_param(&second_param.ty.tokens) {
            Some(MethodInfo(Some(is_mut), None))
        } else {
            None
        }
    // method | static => not yet
    } else {
        if let Some(is_mut) = is_self_param(&func.params[0].0) {
            let (args_ty, return_ty) = get_args_ty(&func);
            Some(MethodInfo(
                Some(is_mut),
                Some((Some(args_ty), Some(return_ty))),
            ))
        } else {
            let (args_ty, return_ty) = get_args_ty(&func);
            Some(MethodInfo(None, Some((Some(args_ty), Some(return_ty)))))
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

// fn gen_raw_method(
//     is_mut: bool,
//     ty_ident: &Ident,
//     fn_ident: &Ident,
//     fn_str: Literal,
// ) -> TokenStream {
//     let ty_self = get_self_ty(is_mut, ty_ident);
//
//     quote! {
//         sys::luaL_Reg {
//             name: cstr!(#fn_str),
//             func: {
//                 unsafe extern "C" fn step(raw_state: *mut sys::lua_State) -> std::ffi::c_int {
//                     let mut state = State::from_raw(raw_state);
//                     let self_mut_ref = state.cast_to::<#ty_self>(1).unwrap();
//                     let n = #ty_ident::#fn_ident(self_mut_ref, &state);
//                     n as std::ffi::c_int
//                 }
//                 Some(step)
//             },
//         }
//     }
// }

// (state: &State)
// fn gen_raw_static(ty_ident: &Ident, fn_ident: &Ident, fn_str: Literal) -> TokenStream {
//     quote! {
//         luajit2_sys::luaL_Reg {
//             name: cstr!(#fn_str),
//             func: {
//                 extern "C" fn step(ptr: *mut sys::lua_State) -> std::ffi::c_int {
//                     let state = State::from_raw(ptr);
//                     <#ty_ident>::#fn_ident(&state) as std::ffi::c_int
//                 }
//                 Some(step)
//             },
//         }
//     }
// }

fn gen_method2(meta: MethodMeta, info: MethodInfo) -> TokenStream {
    let MethodMeta {
        owner_ty,
        method_ty,
        method_str,
    } = meta;

    let step_impl = {
        let mut parts = vec![];
        if info.is_raw() {
            if info.is_method() {
                let self_ty = get_self_ty(info.is_mut(), &owner_ty);
                parts.push(quote! {
                    let self_mut_ref = state.cast_to::<#self_ty>(1).unwrap();
                    <#owner_ty>::#method_ty(self_mut_ref, &state) as std::ffi::c_int
                });
            } else {
                parts.push(quote!(<#owner_ty>::#method_ty(&state) as std::ffi::c_int));
            }
            //
        } else {
            if info.is_method() {
                let self_ty = get_self_ty(info.is_mut(), &owner_ty);
                if info.has_args() {
                    if info.has_return_value() {
                        let (args_ty, result_ty) = {
                            let tuple = info.1.unwrap();
                            (tuple.0.unwrap(), tuple.1.unwrap())
                        };
                        parts.push(quote! {
                            let len = <#args_ty as FromLua>::len() + 1;
                            let idx = len * -1;

                            let ud = state.cast_to::<#self_ty>(idx).unwrap();
                            let args = state.cast_to::<#args_ty>(idx + 1).unwrap();
                            state.push(ud.#method_ty(args));

                            <#result_ty as ToLua>::len() as std::ffi::c_int
                        });
                    } else {
                        let args_ty = info.1.unwrap().0.unwrap();
                        parts.push(quote! {
                            let len = <#args_ty as FromLua>::len() + 1;
                            let idx = len * -1;

                            let ud = state.cast_to::<#self_ty>(idx).unwrap();
                            let args = state.cast_to::<#args_ty>(idx + 1).unwrap();

                            0
                        })
                    }
                } else {
                    if info.has_return_value() {
                        let result_ty = info.1.unwrap().1.unwrap();
                        parts.push(quote! {
                            let ud = state.cast_to::<#self_ty>(-1).unwrap();
                            state.push(ud.#method_ty());

                            <#result_ty as ToLua>::len() as std::ffi::c_int
                        });
                    } else {
                        parts.push(quote! {
                            let ud = state.cast_to::<#self_ty>(-1).unwrap();
                            ud.#method_ty();
                            0
                        });
                    }
                }
            } else {
                // static
                if info.has_args() {
                    if info.has_return_value() {
                        let (args_ty, result_ty) = {
                            let tuple = info.1.unwrap();
                            (tuple.0.unwrap(), tuple.1.unwrap())
                        };
                        parts.push(quote! {
                            let len = <#args_ty as FromLua>::len();
                            let args = state.cast_to::<#args_ty>(len * -1);
                            state.push(<#owner_ty>::#method_ty(args));

                            <#result_ty as ToLua>::len() as std::ffi::c_int
                        });
                    } else {
                        let args_ty = info.1.unwrap().0.unwrap();
                        parts.push(quote! {
                            let len = <#args_ty as FromLua>::len() + 1;
                            let args = state.cast_to::<#args_ty>(len * -1);
                            <#owner_ty>::#method_ty(args);

                            0
                        });
                    }
                } else {
                    if info.has_return_value() {
                        let result_ty = info.1.unwrap().1.unwrap();
                        parts.push(quote! {
                            state.push(<#owner_ty>::#method_ty());
                            <#result_ty as ToLua>::len() as std::ffi::c_int
                        });
                    } else {
                        parts.push(quote!(<#owner_ty>::#method_ty();));
                    }
                }
            }
        }
        quote!(#(#parts)*)
    };
    let step = quote! {
        extern "C" fn step(ptr: *mut sys::lua_State) -> std::ffi::c_int {
            let state = State::from_raw(ptr);
            #step_impl
        }
    };

    quote! {
        luajit2_sys::luaL_Reg {
            name: cstr!(#method_str),
            func: {
                #step
                Some(step)
            }
        }
    }
}

// fn gen_static(
//     ident: &Ident,
//     fn_ident: &Ident,
//     fn_str: Literal,
//     args_ty: TokenStream,
//     return_ty: TokenStream,
// ) -> TokenStream {
//     quote! {
//         luajit2_sys::luaL_Reg {
//             name: cstr!(#fn_str),
//             func: {
//                 extern "C" fn step(ptr: *mut sys::lua_State) -> std::ffi::c_int {
//                     let state = State::from_raw(ptr);
//                     let len = <#args_ty as FromLua>::len();
//                     let idx = len * -1;
//
//                     let args = state.cast_to::<#args_ty>(idx).unwrap();
//                     let result = <#ident>::#fn_ident(args);
//
//                     state.push(result);
//                     <#return_ty as ToLua>::len() as std::ffi::c_int
//                 }
//                 Some(step)
//             },
//         }
//     }
// }

// fn gen_method(
//     is_mut: bool,
//     ident: &Ident,
//     fn_ident: &Ident,
//     fn_str: Literal,
//     args_ty: TokenStream,
//     return_ty: TokenStream,
// ) -> TokenStream {
//     let self_ty = get_self_ty(is_mut, ident);
//     quote! {
//         luajit2_sys::luaL_Reg {
//             name: cstr!(#fn_str),
//             func: {
//                 extern "C" fn step(ptr: *mut sys::lua_State) -> std::ffi::c_int {
//                     let state = State::from_raw(ptr);
//                     let len = <#args_ty as FromLua>::len() + 1;
//                     let idx = len * -1;
//
//                     let ud = state.cast_to::<#self_ty>(idx).unwrap();
//                     let args = state.cast_to::<#args_ty>(idx + 1).unwrap();
//                     let result = ud.#fn_ident(args);
//
//                     state.push(result);
//                     <#return_ty as ToLua>::len() as std::ffi::c_int
//                 }
//                 Some(step)
//             },
//         }
//     }
// }

// fn gen_method_no_arg_no_return(
//     is_mut: bool,
//     ud_ty: &Ident,
//     fn_ident: &Ident,
//     fn_str: Literal,
// ) -> TokenStream {
//     let self_ty = get_self_ty(is_mut, ud_ty);
//     quote! {
//         luajit2_sys::luaL_Reg {
//             name: cstr!(#fn_str),
//             func: {
//                 extern "C" fn step(ptr: *mut sys::lua_State) -> std::ffi::c_int {
//                     let state = State::from_raw(ptr);
//                     let ud = state.cast_to::<#self_ty>(-1).unwrap();
//                     ud.#fn_ident();
//                     0
//                 }
//                 Some(step)
//             },
//         }
//     }
// }

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
                let meta = MethodMeta {
                    owner_ty: ty_ident.clone(),
                    method_ty: fn_ident.clone(),
                    method_str: fn_str,
                };
                decls.push(gen_method2(meta, info));

                // match info {
                //     ParamsInfo::Method(info) => {}
                // ParamsInfo::RawMethod(is_mut) => {
                //     decls.push(gen_raw_method(is_mut, &ty_ident, fn_ident, fn_str))
                // }
                // ParamsInfo::RawStatic => {
                //     decls.push(gen_raw_static(&ty_ident, fn_ident, fn_str))
                // }
                // ParamsInfo::Static(args_ty, return_ty) => {
                //     decls.push(gen_static(&ty_ident, fn_ident, fn_str, args_ty, return_ty))
                // }
                // ParamsInfo::Method(is_mut, args_ty, return_ty) => decls.push(gen_method(
                //     is_mut, &ty_ident, fn_ident, fn_str, args_ty, return_ty,
                // )),
                // ParamsInfo::MethodNoArgNoReturn(is_mut) => decls.push(
                //     gen_method_no_arg_no_return(is_mut, &ty_ident, fn_ident, fn_str),
                // ),
                // }
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
