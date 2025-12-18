use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields};

pub fn input_impl(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = syn::parse::<DeriveInput>(item).unwrap();
    let struct_name = &input.ident;

    let keys_name = syn::Ident::new(
        &format!("{}Keys", struct_name),
        proc_macro2::Span::call_site(),
    );
    let reader_name = syn::Ident::new(
        &format!("{}Reader", struct_name),
        proc_macro2::Span::call_site(),
    );

    let fields_opt = match &input.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(fields_named) => Some(&fields_named.named),
            Fields::Unit => None, // `struct Input;`
            _ => panic!("Only named fields or unit structs are supported"),
        },
        _ => panic!("Only structs are supported"),
    };

    // Unit struct: no keys, reader reads nothing.
    if fields_opt.is_none() {
        let expanded = quote! {
            #[derive(Clone, Debug)]
            #input

            #[::serialization_macros::serializable_struct]
            pub struct #keys_name {}

            pub struct #reader_name;

            impl #reader_name {
                pub fn read(&self) -> #struct_name {
                    #struct_name
                }
            }

            impl ::channels::Reader<#struct_name> for #reader_name {
                fn read(&self) -> #struct_name {
                    #reader_name::read(self)
                }
            }

            impl ::channels::ChannelKeys for #keys_name {
                fn channel_names(&self) -> Vec<String> {
                    vec![]
                }
            }

            impl ::channels::InputKeys<#struct_name> for #keys_name {
                type ReaderType = #reader_name;

                fn reader(&self, _registry: &::channels::ChannelRegistry) -> Result<Self::ReaderType, ::channels::RegistryError> {
                    Ok(#reader_name)
                }
            }

            impl ::block_traits::BlockInput for #struct_name {
                type Keys = #keys_name;
            }
        };
        return TokenStream::from(expanded);
    }

    // Named fields case (existing behavior)
    let fields = fields_opt.unwrap();
    let field_idents = fields.iter().map(|f| f.ident.as_ref().unwrap());

    let key_fields = fields.iter().map(|field| {
        let field_name = &field.ident;
        quote! { pub #field_name: String }
    });

    let reader_fields = fields.iter().map(|field| {
        let field_name = &field.ident;
        let field_type = &field.ty;
        quote! { #field_name: std::rc::Rc<std::cell::RefCell<#field_type>> }
    });

    let reader_assignments = fields.iter().map(|field| {
        let field_name = &field.ident;
        let field_type = &field.ty;
        quote! { #field_name: registry.get::<#field_type>(&self.#field_name)? }
    });

    let read_assignments = fields.iter().map(|field| {
        let field_name = &field.ident;
        quote! { #field_name: *self.#field_name.borrow() }
    });

    let expanded = quote! {
        #[derive(Clone, Debug)]
        #input

        #[::serialization_macros::serializable_struct]
        pub struct #keys_name {
            #(#key_fields,)*
        }

        pub struct #reader_name {
            #(#reader_fields,)*
        }

        impl #reader_name {
            pub fn read(&self) -> #struct_name {
                #struct_name {
                    #(#read_assignments,)*
                }
            }
        }

        impl ::channels::Reader<#struct_name> for #reader_name {
            fn read(&self) -> #struct_name {
                #reader_name::read(self)
            }
        }

        impl ::channels::ChannelKeys for #keys_name {
            fn channel_names(&self) -> Vec<String> {
                vec![ #(self.#field_idents.clone(),)* ]
            }
        }

        impl ::channels::InputKeys<#struct_name> for #keys_name {
            type ReaderType = #reader_name;

            fn reader(&self, registry: &::channels::ChannelRegistry) -> Result<Self::ReaderType, ::channels::RegistryError> {
                Ok(#reader_name { #(#reader_assignments,)* })
            }
        }

        impl ::block_traits::BlockInput for #struct_name {
            type Keys = #keys_name;
        }
    };

    TokenStream::from(expanded)
}
