use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput};

#[proc_macro_derive(Serializable)]
pub fn derive_serializable_struct(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    quote! {
        impl #impl_generics ::serialization::structs::Serializable
            for #name #ty_generics
            #where_clause
        {}
    }
    .into()
}

#[proc_macro_attribute]
pub fn serializable_struct(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let out = quote! {
        #[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
        #input

        impl #impl_generics ::serialization::structs::Serializable
            for #name #ty_generics
            #where_clause
        {}
    };
    out.into()
}

#[proc_macro_attribute]
pub fn serializable_enum(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let name = &input.ident;
    let generics = &input.generics;

    match &input.data {
        Data::Enum(_) => {}
        _ => {
            return syn::Error::new_spanned(
                &input.ident,
                "#[serializable_enum] can only be used on enums",
            )
            .to_compile_error()
            .into();
        }
    }

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    quote! {
        #[derive(
            serde::Serialize,
            serde::Deserialize,
            Clone,
            Debug,
            PartialEq,
            Eq
        )]
        #input

        impl #impl_generics ::serialization::structs::Serializable
            for #name #ty_generics
            #where_clause
        {}
    }
    .into()
}
