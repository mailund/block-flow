use proc_macro::TokenStream;

mod block;
mod contract_deps;
mod execute;
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

/// Make a derive macro for ContractDeps.
/// Will generate an implementation that returns all fields
/// of type Contract. Using the attribute #[no_contract_deps] on a field
/// will skip that field.
#[proc_macro_derive(ContractDeps, attributes(no_contract_deps))]
pub fn contract_deps_derive(item: TokenStream) -> TokenStream {
    contract_deps::contract_deps_impl(item)
}

#[proc_macro_attribute]
pub fn init_params(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = syn::parse::<syn::DeriveInput>(item).unwrap();
    let expanded = quote::quote! {
        #[::serialization_macros::serializable_struct]
        #[derive(::block_macros::ContractDeps)]
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
