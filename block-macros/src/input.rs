use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields};

pub fn input_impl(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = syn::parse::<DeriveInput>(item).unwrap();
    let struct_name = &input.ident;

    // Generate the keys struct name and reader struct name with hygienic names
    let keys_name = syn::Ident::new(
        &format!("{}Keys", struct_name),
        proc_macro2::Span::call_site(),
    );
    let reader_name = syn::Ident::new(
        &format!("{}Reader", struct_name),
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

    // Generate reader fields (all Rc<RefCell<T>> types)
    let reader_fields = fields.iter().map(|field| {
        let field_name = &field.ident;
        let field_type = &field.ty;
        quote! {
            #field_name: std::rc::Rc<std::cell::RefCell<#field_type>>
        }
    });

    // Generate reader method implementation
    let reader_assignments = fields.iter().map(|field| {
        let field_name = &field.ident;
        let field_type = &field.ty;
        quote! {
            #field_name: registry.get::<#field_type>(&self.#field_name)?
        }
    });

    // Generate read method implementation
    let read_assignments = fields.iter().map(|field| {
        let field_name = &field.ident;
        quote! {
            #field_name: *self.#field_name.borrow()
        }
    });

    let expanded = quote! {
        #input

        /// Keys for accessing registry values for #struct_name
        pub struct #keys_name {
            #(#key_fields,)*
        }

        /// Reader that holds direct references to registry values for #struct_name
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

        impl registry::Reader<#struct_name> for #reader_name {
            fn read(&self) -> #struct_name {
                #reader_name::read(self)
            }
        }

        impl registry::InputKeys<#struct_name> for #keys_name {
            type ReaderType = #reader_name;

            fn reader(&self, registry: &registry::Registry) -> Result<Self::ReaderType, registry::RegistryError> {
                Ok(#reader_name {
                    #(#reader_assignments,)*
                })
            }
        }

        impl blocks::BlockInput for #struct_name {
            type Keys = #keys_name;
        }
    };
    TokenStream::from(expanded)
}
