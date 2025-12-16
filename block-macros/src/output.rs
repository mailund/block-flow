use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields};

pub fn output_impl(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = syn::parse::<DeriveInput>(item).unwrap();
    let struct_name = &input.ident;

    // Generate the keys struct name and writer struct name with hygienic names
    let keys_name = syn::Ident::new(
        &format!("{}Keys", struct_name),
        proc_macro2::Span::call_site(),
    );
    let writer_name = syn::Ident::new(
        &format!("{}Writer", struct_name),
        proc_macro2::Span::call_site(),
    );

    // Extract fields from the struct
    let fields = match &input.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(fields_named) => &fields_named.named,
            _ => panic!("Only named fields are supported"),
        },
        _ => panic!("Only structs are supported"),
    };

    // Generate key fields (all String types)
    let key_fields = fields.iter().map(|field| {
        let field_name = &field.ident;
        quote! {
            pub #field_name: String
        }
    });

    // Generate writer fields (all Rc<RefCell<T>> types)
    let writer_fields = fields.iter().map(|field| {
        let field_name = &field.ident;
        let field_type = &field.ty;
        quote! {
            #field_name: std::rc::Rc<std::cell::RefCell<#field_type>>
        }
    });

    // Generate writer method implementation
    let writer_assignments = fields.iter().map(|field| {
        let field_name = &field.ident;
        let field_type = &field.ty;
        quote! {
            #field_name: registry.get::<#field_type>(&self.#field_name)?
        }
    });

    // Generate write method implementation
    let write_assignments = fields.iter().map(|field| {
        let field_name = &field.ident;
        quote! {
            *self.#field_name.borrow_mut() = output.#field_name
        }
    });

    // Extract field names and types for register method
    let field_names: Vec<_> = fields
        .iter()
        .map(|field| field.ident.as_ref().unwrap())
        .collect();
    let field_types: Vec<_> = fields.iter().map(|field| &field.ty).collect();

    let expanded = quote! {
        #input

        /// Keys for accessing registry values for #struct_name
        #[derive(serde::Serialize, serde::Deserialize)]
        pub struct #keys_name {
            #(#key_fields,)*
        }

        /// Writer that holds direct references to registry values for #struct_name
        pub struct #writer_name {
            #(#writer_fields,)*
        }

        impl #writer_name {
            pub fn write(&self, output: &#struct_name) {
                #(#write_assignments;)*
            }
        }

        impl channels::Writer<#struct_name> for #writer_name {
            fn write(&self, output: &#struct_name) {
                #writer_name::write(self, output)
            }
        }

        impl channels::OutputKeys<#struct_name> for #keys_name {
            type WriterType = #writer_name;

            fn writer(&self, registry: &channels::ChannelRegistry) -> Result<Self::WriterType, channels::RegistryError> {
                Ok(#writer_name {
                    #(#writer_assignments,)*
                })
            }

            fn register(&self, registry: &mut channels::ChannelRegistry) {
                #(
                    registry.ensure::<#field_types>(&self.#field_names);
                )*
            }
        }

        impl serialization::structs::SerializableStruct for #keys_name {}

        impl block_traits::BlockOutput for #struct_name {
            type Keys = #keys_name;
        }
    };

    TokenStream::from(expanded)
}
