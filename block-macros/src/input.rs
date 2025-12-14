use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

pub fn input_impl(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let struct_name = &input.ident;

    // Generate the keys struct name and reader struct name
    let keys_name = syn::Ident::new(&format!("{}Keys", struct_name), struct_name.span());
    let reader_name = syn::Ident::new(&format!("{}Reader", struct_name), struct_name.span());

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
            #field_name: registry.get::<#field_type>(&keys.#field_name)?
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

        /// Keys for accessing registry values
        pub struct #keys_name {
            #(#key_fields,)*
        }

        /// Reader that holds direct references to registry values
        pub struct #reader_name {
            #(#reader_fields,)*
        }

        impl #struct_name {
            /// Create a reader from the registry using the provided keys
            pub fn reader(keys: &#keys_name, registry: &registry::Registry) -> Result<#reader_name, registry::RegistryError> {
                Ok(#reader_name {
                    #(#reader_assignments,)*
                })
            }
        }

        impl #reader_name {
            /// Read input values from the captured references
            pub fn read(&self) -> #struct_name {
                #struct_name {
                    #(#read_assignments,)*
                }
            }
        }
    };

    TokenStream::from(expanded)
}
