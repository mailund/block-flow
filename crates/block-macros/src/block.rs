use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Expr, Meta, Path};

pub fn block_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as DeriveInput);
    let struct_name = &input.ident;

    let mut input_type: Option<Path> = None;
    let mut output_type: Option<Path> = None;
    let mut state_type: Option<Path> = None;
    let mut init_type: Option<Path> = None;
    let mut intents_type: Option<Path> = None;

    let mut contract_deps_enabled: Option<bool> = None;

    if !attr.is_empty() {
        let args: syn::punctuated::Punctuated<Meta, syn::Token![,]> =
            syn::parse_macro_input!(attr with syn::punctuated::Punctuated::parse_terminated);

        for meta in args {
            let Meta::NameValue(meta_name_value) = meta else {
                continue;
            };
            let Some(ident) = meta_name_value.path.get_ident() else {
                continue;
            };
            let name = ident.to_string();

            match &meta_name_value.value {
                Expr::Lit(expr_lit) => match &expr_lit.lit {
                    syn::Lit::Str(lit_str) => {
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
                    syn::Lit::Bool(lit_bool) => {
                        if name == "contract_deps" {
                            contract_deps_enabled = Some(lit_bool.value());
                        }
                    }
                    _ => {}
                },
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

    let contract_deps_enabled = contract_deps_enabled.unwrap_or(true);

    let input_type = input_type.unwrap_or_else(|| syn::parse_str("Input").unwrap());
    let output_type = output_type.unwrap_or_else(|| syn::parse_str("Output").unwrap());
    let state_type = state_type.unwrap_or_else(|| syn::parse_str("State").unwrap());
    let init_params = init_type.unwrap_or_else(|| syn::parse_str("InitParams").unwrap());
    let intents_type = intents_type
        .unwrap_or_else(|| syn::parse_str("::block_traits::intents::ZeroIntents").unwrap());

    let mut inject_default_contract_deps_impl = false;

    if contract_deps_enabled {
        let already_has_contract_deps_derive = input.attrs.iter().any(|a| {
            if !a.path().is_ident("derive") {
                return false;
            }
            let Ok(list) = a.parse_args_with(
                syn::punctuated::Punctuated::<Path, syn::Token![,]>::parse_terminated,
            ) else {
                return false;
            };
            list.iter().any(|p| {
                p.segments
                    .last()
                    .map(|s| s.ident == "ContractDeps")
                    .unwrap_or(false)
            })
        });

        if !already_has_contract_deps_derive {
            input
                .attrs
                .push(syn::parse_quote!(#[derive(::block_macros::ContractDeps)]));
        }
    } else {
        inject_default_contract_deps_impl = true;
    }

    let default_contract_deps_impl = if inject_default_contract_deps_impl {
        quote! {
            impl ::block_traits::ContractDeps for #struct_name {
                fn contract_deps(&self) -> Vec<::trade_types::Contract> {
                    Vec::new()
                }
            }
        }
    } else {
        quote! {}
    };

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

        #default_contract_deps_impl
    };

    TokenStream::from(expanded)
}
