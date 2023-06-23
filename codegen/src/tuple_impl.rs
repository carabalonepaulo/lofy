use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

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
                fn to_lua(self, state: *mut sys::lua_State) {
                    let mut state = State::from_raw(state);
                    #(#state_push)*
                }

                fn len() -> i32 { #len }
            }
        });
    });

    quote!(#(#parts)*)
}
