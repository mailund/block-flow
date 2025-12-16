use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields};

pub fn output_impl(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = syn::parse::<DeriveInput>(item).unwrap();
    let struct_name = &input.ident;

    let keys_name = syn::Ident::new(
        &format!("{}Keys", struct_name),
        proc_macro2::Span::call_site(),
    );
    let writer_name = syn::Ident::new(
        &format!("{}Writer", struct_name),
        proc_macro2::Span::call_site(),
    );

    let fields_opt = match &input.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(fields_named) => Some(&fields_named.named),
            Fields::Unit => None, // `struct Output;`
            _ => panic!("Only named fields or unit structs are supported"),
        },
        _ => panic!("Only structs are supported"),
    };

    // Unit struct: generate a Keys type with no fields and a Writer that does nothing.
    if fields_opt.is_none() {
        let expanded = quote! {
            #[derive(Clone, Debug)]
            #input

            #[derive(serde::Serialize, serde::Deserialize, Clone, Debug, Default)]
            pub struct #keys_name {}

            pub struct #writer_name;

            impl #writer_name {
                pub fn write(&self, _output: &#struct_name) {}
            }

            impl channels::Writer<#struct_name> for #writer_name {
                fn write(&self, output: &#struct_name) {
                    #writer_name::write(self, output)
                }
            }

            impl channels::ChannelKeys for #keys_name {
                fn channel_names(&self) -> Vec<String> {
                    vec![]
                }
            }

            impl channels::OutputKeys<#struct_name> for #keys_name {
                type WriterType = #writer_name;

                fn writer(&self, _registry: &channels::ChannelRegistry) -> Result<Self::WriterType, channels::RegistryError> {
                    Ok(#writer_name)
                }

                fn register(&self, _registry: &mut channels::ChannelRegistry) {
                    // no outputs to register
                }
            }

            impl serialization::structs::SerializableStruct for #keys_name {}

            impl block_traits::BlockOutput for #struct_name {
                type Keys = #keys_name;
            }
        };
        return TokenStream::from(expanded);
    }

    // Named fields case (your existing logic)
    let fields = fields_opt.unwrap();
    let field_idents = fields.iter().map(|f| f.ident.as_ref().unwrap());

    let key_fields = fields.iter().map(|field| {
        let field_name = &field.ident;
        quote! { pub #field_name: String }
    });

    let writer_fields = fields.iter().map(|field| {
        let field_name = &field.ident;
        let field_type = &field.ty;
        quote! { #field_name: std::rc::Rc<std::cell::RefCell<#field_type>> }
    });

    let writer_assignments = fields.iter().map(|field| {
        let field_name = &field.ident;
        let field_type = &field.ty;
        quote! { #field_name: registry.get::<#field_type>(&self.#field_name)? }
    });

    let write_assignments = fields.iter().map(|field| {
        let field_name = &field.ident;
        quote! { *self.#field_name.borrow_mut() = output.#field_name.clone() }
    });

    let field_names: Vec<_> = fields.iter().map(|f| f.ident.as_ref().unwrap()).collect();
    let field_types: Vec<_> = fields.iter().map(|f| &f.ty).collect();

    let expanded = quote! {
        #[derive(Clone, Debug)]
        #input

        #[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
        pub struct #keys_name {
            #(#key_fields,)*
        }

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

        impl channels::ChannelKeys for #keys_name {
            fn channel_names(&self) -> Vec<String> {
                vec![ #(self.#field_idents.clone(),)* ]
            }
        }

        impl channels::OutputKeys<#struct_name> for #keys_name {
            type WriterType = #writer_name;

            fn writer(&self, registry: &channels::ChannelRegistry) -> Result<Self::WriterType, channels::RegistryError> {
                Ok(#writer_name { #(#writer_assignments,)* })
            }

            fn register(&self, registry: &mut channels::ChannelRegistry) {
                #( registry.ensure::<#field_types>(&self.#field_names); )*
            }
        }

        impl serialization::structs::SerializableStruct for #keys_name {}

        impl block_traits::BlockOutput for #struct_name {
            type Keys = #keys_name;
        }
    };

    TokenStream::from(expanded)
}
