use proc_macro::TokenStream;

mod block;
// mod block_spec; // Removed - better to write BlockSpec implementations manually
mod init_params;
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
pub fn block(attr: TokenStream, item: TokenStream) -> TokenStream {
    block::block_impl(attr, item)
}

#[proc_macro_attribute]
pub fn init_params(attr: TokenStream, item: TokenStream) -> TokenStream {
    init_params::init_params_impl(attr, item)
}
