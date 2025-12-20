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
    let item = syn::parse::<syn::DeriveInput>(item).unwrap();
    let expanded = quote::quote! {
        #[::serialization_macros::serializable_struct]
        #item
    };
    expanded.into()
}

#[proc_macro_attribute]
pub fn block(attr: TokenStream, item: TokenStream) -> TokenStream {
    block::block_impl(attr, item)
}

// Add a derive that will allow #[no_contract_deps] on fields.
#[proc_macro_derive(InitParamsMarker, attributes(no_contract_deps))]
pub fn init_params_marker_derive(item: TokenStream) -> TokenStream {
    init_params::init_params_impl(item)
}

#[proc_macro_attribute]
pub fn init_params(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = syn::parse::<syn::DeriveInput>(item).unwrap();
    let expanded = quote::quote! {
        #[::serialization_macros::serializable_struct]
        #[derive(::block_macros::InitParamsMarker)]
        #item
    };
    expanded.into()
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
