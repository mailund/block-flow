use proc_macro::TokenStream;

mod block;
mod execute;
mod init_params;
mod input;
mod make_defaults;
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

/// make_defaults!(input, output, init_params, state)
/// make_defaults!(input=MyInput, state=MyState)
#[proc_macro]
pub fn make_defaults(input: TokenStream) -> TokenStream {
    make_defaults::make_defaults_impl(input)
}

/// Optional args:
///   #[execute]
///   #[execute(inner="execute_impl")]   // name for the inner method if we must rename
#[proc_macro_attribute]
pub fn execute(attr: TokenStream, item: TokenStream) -> TokenStream {
    execute::execute_impl(attr, item)
}
