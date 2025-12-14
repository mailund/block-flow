use proc_macro::TokenStream;

mod block_spec;
mod input;
mod output;

#[proc_macro_attribute]
pub fn input(attr: TokenStream, item: TokenStream) -> TokenStream {
    input::input_impl(attr, item)
}

#[proc_macro_attribute]
pub fn output(attr: TokenStream, item: TokenStream) -> TokenStream {
    output::output_impl(attr, item)
}

#[proc_macro_attribute]
pub fn state(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Just a marker for now - return input unchanged
    item
}

#[proc_macro_attribute]
pub fn block(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Just a marker for now - return input unchanged
    item
}

#[proc_macro_attribute]
pub fn block_spec(attr: TokenStream, item: TokenStream) -> TokenStream {
    block_spec::block_spec_impl(attr, item)
}
