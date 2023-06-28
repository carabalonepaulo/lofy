#![allow(unused)]

use proc_macro2::{Ident, Literal, Punct, Spacing, Span, TokenStream, TokenTree};
use quote::quote;
use syn::Index;
use venial::{
    parse_declaration, Declaration, FnParam, FnTypedParam, Function, Impl, ImplMember, Punctuated,
    TyExpr,
};

// - method | (&self, (?: ?)+) -> ?
// - method | (&mut self, (?: ?+)) -> ?
// - raw    | (?: *mut luajit2_sys::lua_State) -> i32
// - static | ((?: ?)+) -> ?

enum ValueKind {
    None,
    Multiple(usize),
    Single,
}

struct Value {
    kind: ValueKind,
    ty: Option<TokenStream>,
}

struct MethodInfo {
    args_ty: Option<Value>,
    result_ty: Value,
}

enum FunctionKind {
    Raw,
    Method(MethodInfo),
}

fn is_raw_func(func: &Function) -> bool {
    if func.params.len() != 1 {
        return false;
    }

    match &func.params[0].0 {
        FnParam::Receiver(param) => false,
        FnParam::Typed(param) => {
            let tokens = &param.ty.tokens;
            let TokenTree::Punct(punct) = &tokens[0] else { return false; };

            if punct.as_char() != '*' {
                false
            } else {
                true
            }
        }
    }
}

fn is_static_func(func: &Function) -> bool {
    false
}

fn is_self_param(param: &FnParam) -> Option<bool> {
    let FnParam::Receiver(first_param) = param else { None? };
    if first_param.tk_ref.is_none() || &first_param.tk_self.to_string().to_lowercase() != "self" {
        None
    } else {
        Some(first_param.tk_mut.is_some())
    }
}

fn match_punct_char(value: &TokenTree, ch: char) -> bool {
    if let TokenTree::Punct(punct) = value {
        punct.as_char() == ch
    } else {
        false
    }
}

fn get_value_kind(tokens: &[TokenTree]) -> ValueKind {
    if match_punct_char(&tokens[0], '(') {
        if match_punct_char(&tokens[1], ')') {
            ValueKind::Single
        } else {
            let inner_paren_count = tokens[1..(tokens.len() - 1)]
                .iter()
                .filter(|tk| match_punct_char(tk, '('))
                .count();
            let comma_count = tokens.iter().filter(|tk| match_punct_char(tk, ',')).count();
            ValueKind::Multiple(comma_count - inner_paren_count)
        }
    } else {
        ValueKind::Single
    }
}

fn parse_args(self_ty: &TyExpr, func: &Function) -> Option<Value> {
    if func.params.len() == 0 {
        return None;
    }

    let mut parts = vec![];

    match &func.params[0].0 {
        FnParam::Receiver(param) => {
            if param.tk_mut.is_some() {
                parts.push(quote!(&mut #self_ty));
            } else {
                parts.push(quote!(&#self_ty));
            }
        }
        FnParam::Typed(param) => {
            let ty = &param.ty;
            parts.push(quote!(#ty));
        }
    }

    let len = func.params.len();
    if len > 1 {
        func.params[1..len]
            .iter()
            .for_each(|(param, punct)| match param {
                FnParam::Receiver(_) => unreachable!(),
                FnParam::Typed(param) => {
                    let ty = &param.ty;
                    parts.push(quote!(#ty))
                }
            });
    }

    let (ty, kind) = if parts.len() > 1 {
        (quote!((#(#parts,)*)), ValueKind::Multiple(parts.len()))
    } else {
        (quote!(#(#parts,)*), ValueKind::Single)
    };

    Some(Value { kind, ty: Some(ty) })
}

fn parse_result(func: &Function) -> Value {
    if func.return_ty.is_none() {
        return Value {
            kind: ValueKind::None,
            ty: None,
        };
    }

    let return_ty = func.return_ty.as_ref().unwrap();
    let return_kind = get_value_kind(&return_ty.tokens);
    Value {
        kind: return_kind,
        ty: Some(quote!(#return_ty)),
    }
}

fn classify(self_ty: &TyExpr, func: &Function) -> Option<FunctionKind> {
    // ignore private functions
    if func.vis_marker.is_none() {
        return None;
    }

    // let args_ty = parse_args(self_ty, func);
    // let result_ty = parse_result(func);
    let info = MethodInfo {
        args_ty: parse_args(self_ty, func),
        result_ty: parse_result(func),
    };

    // parameterless functions can only be static
    if func.params.len() == 0 {
        // return Some(FunctionKind::Method(MethodInfo { args_ty, result_ty }));
        return Some(FunctionKind::Method(info));
    }

    match &func.params[0].0 {
        // (&self)
        // (&self, a: A, b: B)
        // (&self, a: A) -> B
        FnParam::Receiver(param) => Some(FunctionKind::Method(info)),

        // (a: A, b: B, ...) -> ?
        // (state: *mut luajit2_sys::lua_State) -> i32
        FnParam::Typed(param) => {
            if is_raw_func(func) {
                return Some(FunctionKind::Raw);
            }

            Some(FunctionKind::Method(info))
        }
    }
}

fn create_method_call(
    self_ty: &TyExpr,
    fn_ident: &Ident,
    args_ty: &TokenStream,
    args_kind: &ValueKind,
) -> TokenStream {
    let mut parts = vec![];
    match args_kind {
        // só se o filho da puta especificar um parametro como tipo () na definição da função,
        // wtf???
        ValueKind::None => panic!("o que fazer com a porra do nil"),
        ValueKind::Multiple(n) => {
            for i in 0..n.clone() {
                let idx = Index::from(i);
                parts.push(quote!(args.#idx));
            }
        }
        ValueKind::Single => parts.push(quote!(args)),
    }
    quote!(<#self_ty>::#fn_ident(#(#parts,)*))
}

fn try_generate_method(self_ty: &TyExpr, func: &Function) -> Option<TokenStream> {
    let fn_ident = &func.name;
    let kind = classify(self_ty, func)?;
    let middle_part = match kind {
        FunctionKind::Raw => quote! {
            // <#self_ty>::#fn_ident(ptr) as std::ffi::c_int
            0
        },
        FunctionKind::Method(info) => {
            let mut parts = vec![];

            let args = info.args_ty.unwrap();
            let result = info.result_ty;

            let method_call = if let Some(args_ty) = args.ty.as_ref() {
                create_method_call(self_ty, fn_ident, &args_ty, &args.kind)
            } else {
                quote!(<#self_ty>::#fn_ident())
            };

            parts.push(quote!(let state = State::from_raw(ptr);));

            if let Some(args_ty) = args.ty {
                parts.push(quote! {
                    let len = <#args_ty as FromLua>::len();
                    // TODO: unwrap can destroy your life
                    let args = state.cast_to::<#args_ty>(len * -1).unwrap();
                })
            }

            if let Some(result_ty) = result.ty {
                parts.push(quote! {
                    let result = #method_call;
                    state.push(result);
                    <#result_ty as ToLua>::len() as std::ffi::c_int
                });
            } else {
                parts.push(quote! {
                    #method_call;
                    0
                })
            }

            quote!(#(#parts)*)
        }
    };

    let method_str = Literal::string(&func.name.to_string());
    Some(quote! {
        luajit2_sys::luaL_Reg {
            name: cstr!(#method_str),
            func: {
                extern "C" fn step(ptr: *mut sys::lua_State) -> std::ffi::c_int {
                    #middle_part
                }
                Some(step)
            }
        }
    })
}

fn try_parse_impl(item: TokenStream) -> Option<Impl> {
    match parse_declaration(item) {
        Ok(Declaration::Impl(ty)) => Some(ty),
        _ => None, // _ => panic!("user_data attribute can only be used with impl."),
    }
}

pub fn generate_impl(item: TokenStream) -> TokenStream {
    let Some(user_data_impl) = try_parse_impl(item) else { panic!("user_data attribute can only be used with impl.") };
    for item in &user_data_impl.body_items {
        match item {
            ImplMember::Method(func) => {}
            _ => continue,
        }
    }

    let self_ty = &user_data_impl.self_ty;
    let self_ty_str = {
        if let TokenTree::Ident(ident) = &user_data_impl.self_ty.tokens[0] {
            Literal::string(&ident.to_string())
        } else {
            unreachable!()
        }
    };

    let methods: Vec<TokenStream> = user_data_impl
        .body_items
        .iter()
        .filter_map(|item| match item {
            ImplMember::Method(func) => Some(func),
            _ => None,
        })
        .filter_map(|func| try_generate_method(self_ty, func))
        .collect();

    quote! {
        impl UserData for #self_ty {
            fn name() -> *const i8 { cstr!(#self_ty_str) }
            fn functions() -> Vec<sys::luaL_Reg> {
                vec![
                    #(#methods),*
                ]
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use venial::{parse_declaration, Declaration};
}
