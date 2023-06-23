use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote};

fn gen_return_value(last_letter: char) -> TokenStream {
    let mut values = vec![];
    ('A'..=last_letter)
        .map(|c| c.to_lowercase())
        .for_each(|lower| {
            let span = Span::call_site();
            let ident = format_ident!("{}_value", lower.to_string(), span = span);
            values.push(quote!(unsafe { #ident.unwrap_unchecked() }));
        });
    quote! {
        Some((#(#values,)*))
    }
}

fn gen_cast(letter: char) -> TokenStream {
    let uc = letter.to_uppercase();
    let lc = letter.to_lowercase();

    let var_ident = Ident::new(&format!("{}_value", lc), Span::call_site());
    let ty_ident = Ident::new(&uc.to_string(), Span::call_site());

    quote! {
        let #var_ident = state.cast_to::<#ty_ident>(idx);
        if #var_ident.is_none() {
            return None;
        } else {
            idx += 1;
        }
    }
}
/*
#[allow(unused_assignments)]
impl<'a, A, B, C> FromLua<'a> for (A, B, C)
where
A: FromLua<'a, Output = A>,
B: FromLua<'a, Output = B>,
C: FromLua<'a, Output = C>,
{
type Output = (A, B, C);

fn from_lua(state: &State, idx: i32) -> Option<Self::Output> {
    let mut state = state.clone();
    let mut idx = {
        if idx.is_negative() {
            state.get_top() + idx + 1
        } else {
            idx
            }
        };

        if state.get_top() < Self::len() {
            return None;
        }

        let a_value = state.cast_to::<A>(idx);
        if a_value.is_none() {
            return None;
        } else {
            idx += 1;
        }

        let b_value = state.cast_to::<B>(idx);
        if b_value.is_none() {
            return None;
        } else {
            idx += 1;
        }

        let c_value = state.cast_to::<C>(idx);
        if c_value.is_none() {
            return None;
        } else {
            idx += 1;
        }

        Some((
            unsafe { a_value.unwrap_unchecked() },
            unsafe { b_value.unwrap_unchecked() },
            unsafe { c_value.unwrap_unchecked() },
        ))
    }

    fn len() -> i32 {
        3
    }
}
*/
pub fn generate_from_lua_tuple_impl(_: TokenStream) -> TokenStream {
    let max = 25;
    let mut impls = vec![];
    let alphabet: Vec<char> = (b'A'..b'Z').map(|c| c as char).collect();

    (2..max).for_each(|n| {
        let mut letters_a = vec![];
        let mut where_ch = vec![];
        let mut cast_impl = vec![];
        let len = proc_macro2::Literal::i32_unsuffixed(n as i32);

        (0..n).for_each(|i| {
            let letter = alphabet[i];
            let ch = Ident::new(&letter.to_string(), Span::call_site());

            letters_a.push(ch.clone());
            cast_impl.push(gen_cast(letter));
            where_ch.push(quote!(#ch: FromLua<'a, Output = #ch>));
        });

        let return_value = gen_return_value(alphabet[n - 1]);
        let letters_b = letters_a.clone();
        let letters_c = letters_a.clone();
        impls.push(quote! {
            impl<'a, #(#letters_a,)*> FromLua<'a> for (#(#letters_b,)*)
            where
                #(#where_ch,)*
            {
                type Output = (#(#letters_c,)*);

                fn from_lua(ptr: *mut luajit2_sys::lua_State, idx: i32) -> Option<Self::Output> {
                    let mut state = State::from_raw(ptr);
                    let mut idx = {
                        if idx.is_negative() {
                            state.get_top() + idx + 1
                        } else {
                            idx
                        }
                    };

                    if state.get_top() < Self::len() {
                        return None;
                    }

                    #(#cast_impl)*

                    #return_value
                }

                fn len() -> i32 { #len }
            }
        });
    });

    quote!(#(#impls)*)
}

pub fn generate_to_lua_tuple_impl(_attr: TokenStream) -> TokenStream {
    // let max_lit = syn::parse_macro_input!(attr as LitInt);
    // let max = max_lit.base10_parse::<usize>().unwrap() + 1;
    let max = 25;
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
                fn to_lua(self, ptr: *mut luajit2_sys::lua_State) {
                    let mut state = State::from_raw(ptr);
                    #(#state_push)*
                }

                fn len() -> i32 { #len }
            }
        });
    });

    quote!(#(#parts)*)
}
