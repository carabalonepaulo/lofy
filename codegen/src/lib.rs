pub mod tuple_impl;
mod user_data;

use proc_macro2::TokenStream;
use quote::quote;
use venial::{parse_declaration, Declaration};

pub fn generate_user_data_impl(item: TokenStream) -> TokenStream {
    let ty = match parse_declaration(item.clone()) {
        Ok(Declaration::Impl(ty)) => ty,
        _ => panic!("user_data attribute can only be used with impl."),
    };

    let user_data_impl = user_data::gen_user_data_impl(ty);

    quote! {
        #item
        #user_data_impl
    }
}
