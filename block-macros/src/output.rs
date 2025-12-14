use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

pub fn output_impl(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let struct_name = &input.ident;

    // Generate the keys struct name and writer struct name
    let keys_name = syn::Ident::new(&format!("{}Keys", struct_name), struct_name.span());
    let writer_name = syn::Ident::new(&format!("{}Writer", struct_name), struct_name.span());

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

    let expanded = quote! {
        #input

        /// Keys for accessing registry values for #struct_name
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

        impl registry::Writer<#struct_name> for #writer_name {
            fn write(&self, output: &#struct_name) {
                #writer_name::write(self, output)
            }
        }

        impl registry::OutputKeys<#struct_name> for #keys_name {
            type WriterType = #writer_name;

            fn writer(&self, registry: &registry::Registry) -> Result<Self::WriterType, registry::RegistryError> {
                Ok(#writer_name {
                    #(#writer_assignments,)*
                })
            }
        }
    };

    TokenStream::from(expanded)
}
