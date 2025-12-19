use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, spanned::Spanned, AngleBracketedGenericArguments, FnArg, GenericArgument,
    ItemFn, PatType, PathArguments, ReturnType, Type, TypePath,
};

pub fn execute_impl(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut f = parse_macro_input!(item as ItemFn);

    // Save original pieces before rewriting.
    let original_output = f.sig.output.clone();
    let original_block = f.block.clone();

    // Require a receiver (&self / &mut self).
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

    // Keep user-provided args (so their names remain usable in the body).
    let mut context_arg: Option<FnArg> = None;
    let mut input_arg: Option<FnArg> = None;
    let mut state_arg: Option<FnArg> = None;

    for arg in f.sig.inputs.iter() {
        if let FnArg::Typed(PatType { ty, .. }) = arg {
            if is_ref_to(ty, "ExecutionContext") {
                context_arg = Some(arg.clone());
            } else if is_ident_type(ty, "Input") {
                input_arg = Some(arg.clone());
            } else if is_ref_to(ty, "State") {
                state_arg = Some(arg.clone());
            } else {
                return syn::Error::new(
                    ty.span(),
                    "unsupported parameter type for #[execute]. Allowed: &ExecutionContext, Input (by value), &State",
                )
                .to_compile_error()
                .into();
            }
        }
    }

    // Hygienic defaults: `_` binds nothing, so no name can clash with body locals.
    let default_context: FnArg = syn::parse_quote!(_: &::block_traits::ExecutionContext);
    let default_input: FnArg = syn::parse_quote!(
        _: <Self as ::block_traits::BlockSpecAssociatedTypes>::Input
    );
    let default_state: FnArg = syn::parse_quote!(
        _: &<Self as ::block_traits::BlockSpecAssociatedTypes>::State
    );

    let context_arg = context_arg.unwrap_or(default_context);
    let input_arg = input_arg.unwrap_or(default_input);
    let state_arg = state_arg.unwrap_or(default_state);

    // Rewrite signature to full trait signature (fully-qualified).
    f.sig.inputs = {
        let mut inputs = syn::punctuated::Punctuated::new();
        inputs.push(receiver);
        inputs.push(context_arg);
        inputs.push(input_arg);
        inputs.push(state_arg);
        inputs
    };

    // Output is always Option<(Output, State, Intents)>
    f.sig.output = syn::parse_quote!(
        -> ::core::option::Option<(
            <Self as ::block_traits::BlockSpecAssociatedTypes>::Output,
            <Self as ::block_traits::BlockSpecAssociatedTypes>::State,
            <Self as ::block_traits::BlockSpecAssociatedTypes>::Intents
        )>
    );

    let def = quote!(::core::default::Default::default());

    // Build an expression that produces a *non-option* 3-tuple from a value expression.
    // `value_expr` is something like `(|| #original_block )()` OR a binding like `val`.
    fn adapt_value_expr(
        value_expr: proc_macro2::TokenStream,
        ty: &Type,
    ) -> Result<proc_macro2::TokenStream, syn::Error> {
        let def = quote!(::core::default::Default::default());

        // Explicit unit return behaves like "no return type"
        if is_unit_type(ty) {
            return Ok(quote! {
                {
                    let _: () = #value_expr;
                    (#def, #def, #def)
                }
            });
        }

        // Single-value returns (Output / State / Intents)
        if is_output(ty) {
            Ok(quote! {
                {
                    let output: <Self as ::block_traits::BlockSpecAssociatedTypes>::Output = #value_expr;
                    (output, #def, #def)
                }
            })
        } else if is_state(ty) {
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

            // (Output, State)
            if elems.len() == 2 && is_output(elems[0]) && is_state(elems[1]) {
                Ok(quote! {
                    {
                        let (output, state_out): (
                            <Self as ::block_traits::BlockSpecAssociatedTypes>::Output,
                            <Self as ::block_traits::BlockSpecAssociatedTypes>::State
                        ) = #value_expr;
                        (output, state_out, #def)
                    }
                })
            }
            // (Output, Intents)
            else if elems.len() == 2 && is_output(elems[0]) && is_intents(elems[1]) {
                Ok(quote! {
                    {
                        let (output, intents): (
                            <Self as ::block_traits::BlockSpecAssociatedTypes>::Output,
                            <Self as ::block_traits::BlockSpecAssociatedTypes>::Intents
                        ) = #value_expr;
                        (output, #def, intents)
                    }
                })
            }
            // (State, Intents)
            else if elems.len() == 2 && is_state(elems[0]) && is_intents(elems[1]) {
                Ok(quote! {
                    {
                        let (state_out, intents): (
                            <Self as ::block_traits::BlockSpecAssociatedTypes>::State,
                            <Self as ::block_traits::BlockSpecAssociatedTypes>::Intents
                        ) = #value_expr;
                        (#def, state_out, intents)
                    }
                })
            }
            // (Output, State, Intents)
            else if elems.len() == 3
                && is_output(elems[0])
                && is_state(elems[1])
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
                    "unsupported return type for #[execute]. Allowed: Output, State, Intents, (), (Output, State), (Output, Intents), (State, Intents), (Output, State, Intents)",
                ))
            }
        } else {
            Err(syn::Error::new(
                ty.span(),
                "unsupported return type for #[execute]. Allowed: Output, State, Intents, (), (Output, State), (Output, Intents), (State, Intents), (Output, State, Intents)",
            ))
        }
    }

    // Produce final body returning Option<tuple3>.
    let adapted: proc_macro2::TokenStream = match original_output {
        ReturnType::Default => {
            // No explicit return => run body, then Some(defaults)
            quote! {
                (|| #original_block )();
                ::core::option::Option::Some((#def, #def, #def))
            }
        }

        ReturnType::Type(_, ty_box) => {
            let ty: &Type = ty_box.as_ref();

            // Explicit unit return behaves like "no return type"
            if is_unit_type(ty) {
                quote! {
                    (|| #original_block )();
                    ::core::option::Option::Some((#def, #def, #def))
                }
            }
            // If user already returns Option<Inner>, map it.
            else if let Some(inner_ty) = option_inner_type(ty) {
                // Option<()> is allowed and maps to Some(defaults)
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
                // Non-option: compute value, adapt to tuple, wrap Some(...)
                match adapt_value_expr(quote!((|| #original_block )()), ty) {
                    Ok(tuple_expr) => quote! {
                        ::core::option::Option::Some(#tuple_expr)
                    },
                    Err(e) => return e.to_compile_error().into(),
                }
            }
        }
    };

    // Replace body with the adapted one.
    f.block = syn::parse_quote!({ #adapted });

    quote!(#f).into()
}

fn is_ref_to(ty: &Type, name: &str) -> bool {
    matches!(ty, Type::Reference(r) if is_last_segment(&r.elem, name))
}

// Match by last path segment ident == name (works with crate::foo::Input etc.).
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

fn is_ident_type(ty: &Type, name: &str) -> bool {
    is_last_segment(ty, name)
}

fn is_output(ty: &Type) -> bool {
    is_last_segment(ty, "Output")
}

fn is_state(ty: &Type) -> bool {
    is_last_segment(ty, "State")
}

fn is_intents(ty: &Type) -> bool {
    is_last_segment(ty, "Intents")
}

// Explicit unit type `()`
fn is_unit_type(ty: &Type) -> bool {
    matches!(ty, Type::Tuple(tup) if tup.elems.is_empty())
}

// If `ty` is `Option<T>`, return `Some(T)`.
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
