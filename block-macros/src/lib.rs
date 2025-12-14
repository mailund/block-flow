use proc_macro::TokenStream;

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
