use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{spanned::Spanned, Data, DeriveInput, Fields, GenericArgument, PathArguments, Type};

pub fn init_params_impl(item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as DeriveInput);
    let name = input.ident.clone();
    let generics = input.generics.clone();
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let body = match &input.data {
        Data::Struct(s) => match &s.fields {
            Fields::Named(fields) => {
                let mut out = TokenStream2::new();
                for f in &fields.named {
                    if f.attrs
                        .iter()
                        .any(|a| a.path().is_ident("no_contract_deps"))
                    {
                        continue;
                    }
                    let ident = match &f.ident {
                        Some(i) => i,
                        None => continue,
                    };
                    let expr = quote! { &self.#ident };
                    out.extend(emit_collect(&f.ty, expr));
                }
                out
            }
            Fields::Unnamed(_) | Fields::Unit => TokenStream2::new(),
        },
        _ => {
            return syn::Error::new(
                input.span(),
                "InitParamsMarker can only be derived for structs",
            )
            .to_compile_error()
            .into();
        }
    };

    let expanded = quote! {
        impl #impl_generics ::block_traits::ContractDeps for #name #ty_generics #where_clause {
            fn contract_deps(&self) -> ::std::vec::Vec<::trade_types::Contract> {
                let mut deps = ::std::vec::Vec::new();
                #body
                deps
            }
        }
    };

    expanded.into()
}

fn emit_collect(ty: &Type, expr_ref: TokenStream2) -> TokenStream2 {
    match ty {
        Type::Path(tp) => {
            let path = &tp.path;
            let last = match path.segments.last() {
                Some(s) => s,
                None => return TokenStream2::new(),
            };

            if is_contract_path(path) {
                return quote! {
                    deps.push((#expr_ref).clone());
                };
            }

            if last.ident == "Option" {
                let inner = match single_generic_type(path) {
                    Some(t) => t,
                    None => return TokenStream2::new(),
                };
                let inner_expr = quote! { v };
                let inner_tokens = emit_collect(inner, inner_expr);
                return quote! {
                    if let ::std::option::Option::Some(v) = (#expr_ref).as_ref() {
                        #inner_tokens
                    }
                };
            }

            if last.ident == "Vec" {
                let inner = match single_generic_type(path) {
                    Some(t) => t,
                    None => return TokenStream2::new(),
                };
                let inner_expr = quote! { v };
                let inner_tokens = emit_collect(inner, inner_expr);
                return quote! {
                    for v in (#expr_ref).iter() {
                        #inner_tokens
                    }
                };
            }

            TokenStream2::new()
        }
        _ => TokenStream2::new(),
    }
}

fn is_contract_path(path: &syn::Path) -> bool {
    path.segments.last().is_some_and(|s| s.ident == "Contract")
}

fn single_generic_type(path: &syn::Path) -> Option<&Type> {
    let seg = path.segments.last()?;
    match &seg.arguments {
        PathArguments::AngleBracketed(ab) if ab.args.len() == 1 => match ab.args.first()? {
            GenericArgument::Type(t) => Some(t),
            _ => None,
        },
        _ => None,
    }
}
