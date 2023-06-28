pub mod tuple_impl;
mod user_data;
mod user_data2;

use proc_macro2::TokenStream;

pub fn generate_user_data_impl(item: TokenStream) -> TokenStream {
    user_data2::generate_impl(item)
    // let ty = match parse_declaration(item.clone()) {
    //     Ok(Declaration::Impl(ty)) => ty,
    //     _ => panic!("user_data attribute can only be used with impl."),
    // };
    //
    // let user_data_impl = user_data::gen_user_data_impl(ty);
    //
    // quote! {
    //     #item
    //     #user_data_impl
    // }
}
