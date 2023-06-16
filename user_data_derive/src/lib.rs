use proc_macro::TokenStream;
use quote::*;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(UserData)]
pub fn user_data_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let struct_name = input.ident;

    let trait_impl = quote! {
        impl UserData for #struct_name {
            fn name() -> *const i8 {
                cstr!("Test")
            }

            fn functions() -> Vec<sys::luaL_Reg> {
                vec![
                    // lua_func!(#struct_name, #struct_name::new, "new"),
                    // lua_method!(#struct_name, #struct_name::foo, "foo"),
                ]
            }
        }
    };

    trait_impl.to_token_stream().into()
}

#[proc_macro_attribute]
pub fn user_data(attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn ctor(attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn method(attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}
