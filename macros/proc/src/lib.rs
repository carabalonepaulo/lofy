use proc_macro::{self, TokenStream};

#[proc_macro]
pub fn generate_to_lua_tuple_impl(attr: TokenStream) -> TokenStream {
    codegen::tuple_impl::generate_to_lua_tuple_impl(attr.into()).into()
}

#[proc_macro]
pub fn generate_from_lua_tuple_impl(attr: TokenStream) -> TokenStream {
    codegen::tuple_impl::generate_from_lua_tuple_impl(attr.into()).into()
}

#[proc_macro_attribute]
pub fn user_data(_attr: TokenStream, item: TokenStream) -> TokenStream {
    codegen::generate_user_data_impl(item.into()).into()
}
