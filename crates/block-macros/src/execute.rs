use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, spanned::Spanned, AngleBracketedGenericArguments, FnArg, GenericArgument,
    ItemFn, Pat, PatType, PathArguments, ReturnType, Type, TypePath,
};

pub fn execute_impl(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut f = parse_macro_input!(item as ItemFn);

    let original_output = f.sig.output.clone();
    let original_block = f.block.clone();

    let receiver = f.sig.inputs.iter().find_map(|a| match a {
        FnArg::Receiver(r) => Some(FnArg::Receiver(r.clone())),
        _ => None,
    });
    let Some(receiver) = receiver else {
        return syn::Error::new(
            f.sig.ident.span(),
            "#[execute] methods must have a receiver (e.g. &self)",
        )
        .to_compile_error()
        .into();
    };

    let mut ctx_pat: Option<Box<Pat>> = None;
    let mut input_pat: Option<Box<Pat>> = None;
    let mut state_pat: Option<Box<Pat>> = None;

    for arg in f.sig.inputs.iter() {
        if let FnArg::Typed(PatType { pat, ty, .. }) = arg {
            if ctx_pat.is_none() && (is_ref_to_exec_ctx_trait(ty) || is_ref_to_generic_param(ty)) {
                ctx_pat = Some(pat.clone());
                continue;
            }
            if input_pat.is_none() && is_input_value(ty) {
                input_pat = Some(pat.clone());
                continue;
            }
            if state_pat.is_none() && is_ref_to_state(ty) {
                state_pat = Some(pat.clone());
                continue;
            }
            return syn::Error::new(
                ty.span(),
                "unsupported parameter type for #[execute]. Allowed: &ExeContext (generic), Input (by value), &State",
            )
            .to_compile_error()
            .into();
        }
    }

    let ctx_pat: Pat = ctx_pat.map(|p| *p).unwrap_or_else(|| syn::parse_quote!(_));
    let input_pat: Pat = input_pat
        .map(|p| *p)
        .unwrap_or_else(|| syn::parse_quote!(_));
    let state_pat: Pat = state_pat
        .map(|p| *p)
        .unwrap_or_else(|| syn::parse_quote!(_));

    f.sig.generics = syn::parse_quote!(<ExeContext: ::block_traits::ExecutionContextTrait>);

    f.sig.inputs = {
        let mut inputs = syn::punctuated::Punctuated::new();
        inputs.push(receiver);
        inputs.push(syn::parse_quote!(#ctx_pat: &ExeContext));
        inputs.push(syn::parse_quote!(
            #input_pat: <Self as ::block_traits::BlockSpecAssociatedTypes>::Input
        ));
        inputs.push(syn::parse_quote!(
            #state_pat: &<Self as ::block_traits::BlockSpecAssociatedTypes>::State
        ));
        inputs
    };

    f.sig.output = syn::parse_quote!(
        -> ::core::option::Option<(
            <Self as ::block_traits::BlockSpecAssociatedTypes>::Output,
            <Self as ::block_traits::BlockSpecAssociatedTypes>::State,
            <Self as ::block_traits::BlockSpecAssociatedTypes>::Intents
        )>
    );

    let def = quote!(::core::default::Default::default());

    fn adapt_value_expr(
        value_expr: proc_macro2::TokenStream,
        ty: &Type,
    ) -> Result<proc_macro2::TokenStream, syn::Error> {
        let def = quote!(::core::default::Default::default());

        if is_unit_type(ty) {
            return Ok(quote! {
                {
                    let _: () = #value_expr;
                    (#def, #def, #def)
                }
            });
        }

        if is_output(ty) {
            Ok(quote! {
                {
                    let output: <Self as ::block_traits::BlockSpecAssociatedTypes>::Output = #value_expr;
                    (output, #def, #def)
                }
            })
        } else if is_state_value(ty) {
            Ok(quote! {
                {
                    let state_out: <Self as ::block_traits::BlockSpecAssociatedTypes>::State = #value_expr;
                    (#def, state_out, #def)
                }
            })
        } else if is_intents(ty) {
            Ok(quote! {
                {
                    let intents: <Self as ::block_traits::BlockSpecAssociatedTypes>::Intents = #value_expr;
                    (#def, #def, intents)
                }
            })
        } else if let Type::Tuple(tup) = ty {
            let elems: Vec<&Type> = tup.elems.iter().collect();

            if elems.len() == 2 && is_output(elems[0]) && is_state_value(elems[1]) {
                Ok(quote! {
                    {
                        let (output, state_out): (
                            <Self as ::block_traits::BlockSpecAssociatedTypes>::Output,
                            <Self as ::block_traits::BlockSpecAssociatedTypes>::State
                        ) = #value_expr;
                        (output, state_out, #def)
                    }
                })
            } else if elems.len() == 2 && is_output(elems[0]) && is_intents(elems[1]) {
                Ok(quote! {
                    {
                        let (output, intents): (
                            <Self as ::block_traits::BlockSpecAssociatedTypes>::Output,
                            <Self as ::block_traits::BlockSpecAssociatedTypes>::Intents
                        ) = #value_expr;
                        (output, #def, intents)
                    }
                })
            } else if elems.len() == 2 && is_state_value(elems[0]) && is_intents(elems[1]) {
                Ok(quote! {
                    {
                        let (state_out, intents): (
                            <Self as ::block_traits::BlockSpecAssociatedTypes>::State,
                            <Self as ::block_traits::BlockSpecAssociatedTypes>::Intents
                        ) = #value_expr;
                        (#def, state_out, intents)
                    }
                })
            } else if elems.len() == 3
                && is_output(elems[0])
                && is_state_value(elems[1])
                && is_intents(elems[2])
            {
                Ok(quote! {
                    {
                        let (output, state_out, intents): (
                            <Self as ::block_traits::BlockSpecAssociatedTypes>::Output,
                            <Self as ::block_traits::BlockSpecAssociatedTypes>::State,
                            <Self as ::block_traits::BlockSpecAssociatedTypes>::Intents
                        ) = #value_expr;
                        (output, state_out, intents)
                    }
                })
            } else {
                Err(syn::Error::new(
                    tup.span(),
                    "unsupported return type for #[execute]",
                ))
            }
        } else {
            Err(syn::Error::new(
                ty.span(),
                "unsupported return type for #[execute]",
            ))
        }
    }

    let adapted: proc_macro2::TokenStream = match original_output {
        ReturnType::Default => quote! {
            (|| #original_block )();
            ::core::option::Option::Some((#def, #def, #def))
        },
        ReturnType::Type(_, ty_box) => {
            let ty: &Type = ty_box.as_ref();

            if is_unit_type(ty) {
                quote! {
                    (|| #original_block )();
                    ::core::option::Option::Some((#def, #def, #def))
                }
            } else if let Some(inner_ty) = option_inner_type(ty) {
                if is_unit_type(inner_ty) {
                    quote! {
                        match (|| #original_block )() {
                            ::core::option::Option::Some(()) => ::core::option::Option::Some((#def, #def, #def)),
                            ::core::option::Option::None => ::core::option::Option::None,
                        }
                    }
                } else {
                    match adapt_value_expr(quote!(val), inner_ty) {
                        Ok(tuple_expr) => quote! {
                            match (|| #original_block )() {
                                ::core::option::Option::Some(val) => ::core::option::Option::Some(#tuple_expr),
                                ::core::option::Option::None => ::core::option::Option::None,
                            }
                        },
                        Err(e) => return e.to_compile_error().into(),
                    }
                }
            } else {
                match adapt_value_expr(quote!((|| #original_block )()), ty) {
                    Ok(tuple_expr) => quote! {
                        ::core::option::Option::Some(#tuple_expr)
                    },
                    Err(e) => return e.to_compile_error().into(),
                }
            }
        }
    };

    f.block = syn::parse_quote!({ #adapted });

    quote!(#f).into()
}

fn is_ref_to_exec_ctx_trait(ty: &Type) -> bool {
    matches!(ty, Type::Reference(r) if is_last_segment(&r.elem, "ExecutionContextTrait"))
}

fn is_ref_to_state(ty: &Type) -> bool {
    matches!(ty, Type::Reference(r) if is_last_segment(&r.elem, "State"))
}

fn is_ref_to_generic_param(ty: &Type) -> bool {
    let Type::Reference(r) = ty else { return false };
    let Type::Path(TypePath { qself: None, path }) = r.elem.as_ref() else {
        return false;
    };
    path.segments.len() == 1
}

fn is_input_value(ty: &Type) -> bool {
    is_last_segment(ty, "Input")
}

fn is_output(ty: &Type) -> bool {
    is_last_segment(ty, "Output")
}

fn is_state_value(ty: &Type) -> bool {
    is_last_segment(ty, "State")
}

fn is_intents(ty: &Type) -> bool {
    is_last_segment(ty, "Intents")
}

fn is_unit_type(ty: &Type) -> bool {
    matches!(ty, Type::Tuple(tup) if tup.elems.is_empty())
}

fn is_last_segment(ty: &Type, name: &str) -> bool {
    match ty {
        Type::Path(p) => p
            .path
            .segments
            .last()
            .map(|s| s.ident == name)
            .unwrap_or(false),
        _ => false,
    }
}

fn option_inner_type(ty: &Type) -> Option<&Type> {
    let Type::Path(TypePath { qself: None, path }) = ty else {
        return None;
    };
    let seg = path.segments.last()?;
    if seg.ident != "Option" {
        return None;
    }
    let PathArguments::AngleBracketed(AngleBracketedGenericArguments { args, .. }) = &seg.arguments
    else {
        return None;
    };
    if args.len() != 1 {
        return None;
    }
    match args.first()? {
        GenericArgument::Type(t) => Some(t),
        _ => None,
    }
}
