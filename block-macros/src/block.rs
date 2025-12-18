use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Expr, Meta, Path};

pub fn block_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let struct_name = &input.ident;

    let mut input_type: Option<Path> = None;
    let mut output_type: Option<Path> = None;
    let mut state_type: Option<Path> = None;
    let mut init_type: Option<Path> = None;
    let mut intents_type: Option<Path> = None;

    // Parse the attributes if provided
    if !attr.is_empty() {
        // Parse the attributes using syn's proper parser
        let args: syn::punctuated::Punctuated<Meta, syn::Token![,]> =
            syn::parse_macro_input!(attr with syn::punctuated::Punctuated::parse_terminated);

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
                                "init" => init_type = Some(type_path),
                                "intents" => intents_type = Some(type_path),
                                _ => {}
                            }
                        }
                    }
                    Expr::Path(expr_path) => match name.as_str() {
                        "input" => input_type = Some(expr_path.path.clone()),
                        "output" => output_type = Some(expr_path.path.clone()),
                        "state" => state_type = Some(expr_path.path.clone()),
                        "init" => init_type = Some(expr_path.path.clone()),
                        "intents" => intents_type = Some(expr_path.path.clone()),
                        _ => {}
                    },
                    _ => {}
                }
            }
        }
    }

    // Use defaults if not specified
    let input_type = input_type.unwrap_or_else(|| syn::parse_str("Input").unwrap());
    let output_type = output_type.unwrap_or_else(|| syn::parse_str("Output").unwrap());
    let state_type = state_type.unwrap_or_else(|| syn::parse_str("State").unwrap());
    let init_params = init_type.unwrap_or_else(|| syn::parse_str("InitParams").unwrap());
    let intents_type =
        intents_type.unwrap_or_else(|| syn::parse_str("::intents::ZeroIntents").unwrap());

    let expanded = quote! {
        #[derive(Clone, Debug)]
        #input

        impl ::block_traits::BlockSpecAssociatedTypes for #struct_name {
            type Input = #input_type;
            type Output = #output_type;
            type State = #state_type;
            type InitParameters = #init_params;
            type Intents = #intents_type;
        }
    };

    TokenStream::from(expanded)
}
