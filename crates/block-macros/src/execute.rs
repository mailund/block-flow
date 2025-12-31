use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, spanned::Spanned, AngleBracketedGenericArguments, FnArg, GenericArgument,
    ItemFn, Pat, PatType, PathArguments, ReturnType, Type, TypePath,
};

/// Attribute macro implementation for rewriting `execute` methods into a uniform signature
/// and a uniform `Result<(Output, State, Intents), FailureStatus>` return type.
///
/// This macro:
/// - Normalizes the method signature to always include `context`, `input`, `state`, `effect_consumer`
/// - Inserts the generic parameters `<C, E>` with the required trait bounds
/// - Adapts various original return shapes (unit/output/state/intents/tuples/Option/Result)
///   into the canonical `(Output, State, Intents)` tuple wrapped in `Result`.
pub fn execute_impl(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the annotated function.
    let mut f = parse_macro_input!(item as ItemFn);

    // Save the original return type and function body so they can be adapted later.
    let original_output = f.sig.output.clone();
    let original_block = f.block.clone();

    // Extract the receiver (`self`, `&self`, or `&mut self`). This macro requires a receiver.
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

    // Collect patterns for the supported parameters. The macro supports a flexible input
    // signature and will fill in missing parameters with `_`.
    let mut ctx_pat: Option<Box<Pat>> = None;
    let mut input_pat: Option<Box<Pat>> = None;
    let mut state_pat: Option<Box<Pat>> = None;
    let mut eff_pat: Option<Box<Pat>> = None;

    // Walk the input argument list and classify each typed argument into one of:
    // - context: `&C` where `C: ExecutionContextTrait` (or a generic single ident)
    // - input: `Input` (by value)
    // - state: `&State`
    // - effect consumer: `&mut E` where `E: EffectConsumerTrait` (or a generic single ident)
    //
    // Any other parameter shape is rejected to keep the rewriting predictable.
    for arg in f.sig.inputs.iter() {
        if let FnArg::Typed(PatType { pat, ty, .. }) = arg {
            if ctx_pat.is_none() && is_ref_to_exec_ctx_trait_or_generic(ty) {
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
            if eff_pat.is_none() && is_mut_ref_to_effect_consumer_trait_or_generic(ty) {
                eff_pat = Some(pat.clone());
                continue;
            }

            return syn::Error::new(
                ty.span(),
                "unsupported parameter type for #[execute]. Allowed: &C (ExecutionContextTrait), Input (by value), &State, &mut E (EffectConsumerTrait)",
            )
            .to_compile_error()
            .into();
        }
    }

    // Use the collected patterns if present; otherwise use `_` so callers can omit
    // any of the supported parameters from their source signature.
    let ctx_pat: Pat = ctx_pat.map(|p| *p).unwrap_or_else(|| syn::parse_quote!(_));
    let input_pat: Pat = input_pat
        .map(|p| *p)
        .unwrap_or_else(|| syn::parse_quote!(_));
    let state_pat: Pat = state_pat
        .map(|p| *p)
        .unwrap_or_else(|| syn::parse_quote!(_));
    let eff_pat: Pat = eff_pat.map(|p| *p).unwrap_or_else(|| syn::parse_quote!(_));

    // Force generics to the canonical `<C, E>` form with the required trait bounds.
    // This guarantees the generated method matches the trait method signature.
    f.sig.generics = syn::parse_quote!(
        <C: ::block_traits::ExecutionContextTrait, E: ::block_traits::EffectConsumerTrait>
    );

    // Rewrite the function arguments into the canonical order and types.
    // - receiver
    // - &C context
    // - Input (associated type)
    // - &State (associated type)
    // - &mut E effect consumer
    f.sig.inputs = {
        let mut inputs = syn::punctuated::Punctuated::new();
        inputs.push(receiver);
        inputs.push(syn::parse_quote!(#ctx_pat: &C));
        inputs.push(syn::parse_quote!(
            #input_pat: <Self as ::block_traits::BlockSpecAssociatedTypes>::Input
        ));
        inputs.push(syn::parse_quote!(
            #state_pat: &<Self as ::block_traits::BlockSpecAssociatedTypes>::State
        ));
        inputs.push(syn::parse_quote!(#eff_pat: &mut E));
        inputs
    };

    // Force the return type to the canonical result shape.
    // The body adaptation logic will convert the original return expression into this.
    f.sig.output = syn::parse_quote!(
        -> ::core::result::Result<(
            <Self as ::block_traits::BlockSpecAssociatedTypes>::Output,
            <Self as ::block_traits::BlockSpecAssociatedTypes>::State,
            <Self as ::block_traits::BlockSpecAssociatedTypes>::Intents
        ), ::block_traits::execute_status::FailureStatus>
    );

    // Defaults are used when the original method returns only a subset of the
    // canonical tuple components.
    let def = quote!(::core::default::Default::default());

    // Failure value used when translating `Option::None` to an error.
    let fail = quote!(::block_traits::execute_status::FailureStatus::Failure);

    /// Given an expression of some return type `ty`, build an expression that yields
    /// the canonical `(Output, State, Intents)` tuple.
    ///
    /// Supported inputs:
    /// - `()` -> (default, default, default)
    /// - `Output` -> (output, default, default)
    /// - `State` -> (default, state, default)
    /// - `Intents` -> (default, default, intents)
    /// - `(Output, State)` / `(Output, Intents)` / `(State, Intents)` / `(Output, State, Intents)`
    fn adapt_ok_tuple_expr(
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

    // Rewrite the original function body to produce `Result<(Output, State, Intents), FailureStatus>`.
    //
    // Supported original return shapes:
    // - no explicit return / `()` / `Result<(), _>` / `Option<()>`
    // - `Result<T, FailureStatus>`: map Ok(T) into the canonical tuple, propagate Err(_)
    // - `Option<T>`: map Some(T) into Ok(tuple), map None into Err(Failure)
    // - plain `T`: wrap into Ok(tuple)
    let adapted: proc_macro2::TokenStream = match original_output {
        ReturnType::Default => quote! {
            (|| #original_block )();
            ::core::result::Result::Ok((#def, #def, #def))
        },
        ReturnType::Type(_, ty_box) => {
            let ty: &Type = ty_box.as_ref();

            if is_unit_type(ty) {
                quote! {
                    (|| #original_block )();
                    ::core::result::Result::Ok((#def, #def, #def))
                }
            } else if let Some(ok_ty) = result_ok_inner_type(ty) {
                if is_unit_type(ok_ty) {
                    quote! {
                        match (|| #original_block )() {
                            ::core::result::Result::Ok(()) => ::core::result::Result::Ok((#def, #def, #def)),
                            ::core::result::Result::Err(e) => ::core::result::Result::Err(e),
                        }
                    }
                } else {
                    match adapt_ok_tuple_expr(quote!(val), ok_ty) {
                        Ok(tuple_expr) => quote! {
                            match (|| #original_block )() {
                                ::core::result::Result::Ok(val) => ::core::result::Result::Ok(#tuple_expr),
                                ::core::result::Result::Err(e) => ::core::result::Result::Err(e),
                            }
                        },
                        Err(e) => return e.to_compile_error().into(),
                    }
                }
            } else if let Some(inner_ty) = option_inner_type(ty) {
                if is_unit_type(inner_ty) {
                    quote! {
                        match (|| #original_block )() {
                            ::core::option::Option::Some(()) => ::core::result::Result::Ok((#def, #def, #def)),
                            ::core::option::Option::None => ::core::result::Result::Err(#fail),
                        }
                    }
                } else {
                    match adapt_ok_tuple_expr(quote!(val), inner_ty) {
                        Ok(tuple_expr) => quote! {
                            match (|| #original_block )() {
                                ::core::option::Option::Some(val) => ::core::result::Result::Ok(#tuple_expr),
                                ::core::option::Option::None => ::core::result::Result::Err(#fail),
                            }
                        },
                        Err(e) => return e.to_compile_error().into(),
                    }
                }
            } else {
                match adapt_ok_tuple_expr(quote!((|| #original_block )()), ty) {
                    Ok(tuple_expr) => quote! {
                        ::core::result::Result::Ok(#tuple_expr)
                    },
                    Err(e) => return e.to_compile_error().into(),
                }
            }
        }
    };

    // Replace the function body with the adapted body.
    f.block = syn::parse_quote!({ #adapted });

    // Emit the rewritten function.
    quote!(#f).into()
}

/// Returns true if `ty` is an immutable reference to a type that is either:
/// - named `ExecutionContextTrait` (by last segment), or
/// - a single-identifier type (treated as a generic context type).
fn is_ref_to_exec_ctx_trait_or_generic(ty: &Type) -> bool {
    matches!(ty, Type::Reference(r)
        if r.mutability.is_none()
            && (is_last_segment(&r.elem, "ExecutionContextTrait") || is_ref_to_single_ident(&r.elem)))
}

/// Returns true if `ty` is a mutable reference to a type that is either:
/// - named `EffectConsumerTrait` (by last segment), or
/// - a single-identifier type (treated as a generic effect consumer type).
fn is_mut_ref_to_effect_consumer_trait_or_generic(ty: &Type) -> bool {
    matches!(ty, Type::Reference(r)
        if r.mutability.is_some()
            && (is_last_segment(&r.elem, "EffectConsumerTrait") || is_ref_to_single_ident(&r.elem)))
}

/// Returns true if `ty` is an immutable reference whose element type ends with `State`.
fn is_ref_to_state(ty: &Type) -> bool {
    matches!(ty, Type::Reference(r) if r.mutability.is_none() && is_last_segment(&r.elem, "State"))
}

/// Returns true if `ty` is a simple path type with a single identifier and no generic arguments.
fn is_ref_to_single_ident(ty: &Type) -> bool {
    let Type::Path(TypePath { qself: None, path }) = ty else {
        return false;
    };
    path.segments.len() == 1 && matches!(path.segments[0].arguments, PathArguments::None)
}

/// Returns true if `ty` ends with `Input` (by last path segment).
fn is_input_value(ty: &Type) -> bool {
    is_last_segment(ty, "Input")
}

/// Returns true if `ty` ends with `Output` (by last path segment).
fn is_output(ty: &Type) -> bool {
    is_last_segment(ty, "Output")
}

/// Returns true if `ty` ends with `State` (by last path segment).
fn is_state_value(ty: &Type) -> bool {
    is_last_segment(ty, "State")
}

/// Returns true if `ty` ends with `Intents` (by last path segment).
fn is_intents(ty: &Type) -> bool {
    is_last_segment(ty, "Intents")
}

/// Returns true if `ty` is the unit type `()`.
fn is_unit_type(ty: &Type) -> bool {
    matches!(ty, Type::Tuple(tup) if tup.elems.is_empty())
}

/// Returns true if `ty` is a path type whose last segment identifier matches `name`.
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

/// If `ty` is `Option<T>`, returns `Some(T)`. Otherwise returns `None`.
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

/// If `ty` is `Result<T, E>`, returns `Some(T)` (the Ok type). Otherwise returns `None`.
fn result_ok_inner_type(ty: &Type) -> Option<&Type> {
    let Type::Path(TypePath { qself: None, path }) = ty else {
        return None;
    };
    let seg = path.segments.last()?;
    if seg.ident != "Result" {
        return None;
    }
    let PathArguments::AngleBracketed(AngleBracketedGenericArguments { args, .. }) = &seg.arguments
    else {
        return None;
    };
    if args.len() != 2 {
        return None;
    }
    match args.first()? {
        GenericArgument::Type(t) => Some(t),
        _ => None,
    }
}
