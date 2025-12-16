use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemStruct};

pub fn init_params_impl(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut s = parse_macro_input!(item as ItemStruct);
    let name = s.ident.clone();

    // Prepend derives (simple + reliable; does not attempt to merge)
    s.attrs.insert(
        0,
        syn::parse_quote!(#[derive(serde::Serialize, serde::Deserialize)]),
    );

    let expanded = quote! {
        #s
        impl ::serialization::structs::SerializableStruct for #name {}
    };

    expanded.into()
}
