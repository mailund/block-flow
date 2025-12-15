use proc_macro::TokenStream;

pub fn block_impl(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // The #[block] attribute is just a marker for the #[block_spec] macro
    // to identify which struct represents the block. It doesn't generate any code.
    item
}
