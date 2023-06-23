#[proc_macro]
pub fn generate_to_lua_tuple_impl(attr: proc_macro::TokenStream) -> proc_macro::TokenStream {
    codegen::tuple_impl::generate_to_lua_tuple_impl(attr.into()).into()
}

#[proc_macro_attribute]
pub fn user_data(
    _attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    codegen::generate_user_data_impl(item.into()).into()
}
