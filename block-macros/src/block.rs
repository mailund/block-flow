use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Expr, Meta, Path};

pub fn block_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let struct_name = &input.ident;

    // Check if no attributes provided
    if attr.is_empty() {
        let expanded = quote! {
            #input
            compile_error!("Missing required arguments to #[block] macro: input, output, state");
        };
        return TokenStream::from(expanded);
    }

    // Parse the attributes using syn's proper parser
    let args: syn::punctuated::Punctuated<Meta, syn::Token![,]> =
        syn::parse_macro_input!(attr with syn::punctuated::Punctuated::parse_terminated);

    let mut input_type: Option<Path> = None;
    let mut output_type: Option<Path> = None;
    let mut state_type: Option<Path> = None;

    for meta in args {
        if let Meta::NameValue(meta_name_value) = meta {
            let name = meta_name_value.path.get_ident().unwrap().to_string();

            // Handle both string literals and identifiers
            match &meta_name_value.value {
                Expr::Lit(expr_lit) => {
                    if let syn::Lit::Str(lit_str) = &expr_lit.lit {
                        let type_path: Path = syn::parse_str(&lit_str.value()).unwrap();
                        match name.as_str() {
                            "input" => input_type = Some(type_path),
                            "output" => output_type = Some(type_path),
                            "state" => state_type = Some(type_path),
                            _ => {}
                        }
                    }
                }
                Expr::Path(expr_path) => match name.as_str() {
                    "input" => input_type = Some(expr_path.path.clone()),
                    "output" => output_type = Some(expr_path.path.clone()),
                    "state" => state_type = Some(expr_path.path.clone()),
                    _ => {}
                },
                _ => {}
            }
        }
    }

    // Check if required arguments are missing
    let mut missing_args = Vec::new();
    if input_type.is_none() {
        missing_args.push("input");
    }
    if output_type.is_none() {
        missing_args.push("output");
    }
    if state_type.is_none() {
        missing_args.push("state");
    }

    if !missing_args.is_empty() {
        let error_msg = format!(
            "Missing required arguments to #[block] macro: {}",
            missing_args.join(", ")
        );
        let expanded = quote! {
            #input
            compile_error!(#error_msg);
        };
        return TokenStream::from(expanded);
    }

    let input_type = input_type.unwrap();
    let output_type = output_type.unwrap();
    let state_type = state_type.unwrap();

    let expanded = quote! {
        #input

        impl blocks::BlockSpecAssociatedTypes for #struct_name {
            type Input = #input_type;
            type Output = #output_type;
            type State = #state_type;
        }
    };

    TokenStream::from(expanded)
}
