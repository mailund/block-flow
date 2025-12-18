use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, spanned::Spanned, FnArg, ItemFn, PatType, ReturnType, Type};

pub fn execute_impl(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut f = parse_macro_input!(item as ItemFn);

    // Save original pieces before rewriting.
    let original_output = f.sig.output.clone();
    let original_block = (*f.block).clone();

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

    f.sig.output = syn::parse_quote!(
        -> (
            <Self as ::block_traits::BlockSpecAssociatedTypes>::Output,
            <Self as ::block_traits::BlockSpecAssociatedTypes>::State,
            <Self as ::block_traits::BlockSpecAssociatedTypes>::Intents
        )
    );

    // Helpers for defaults (fully-qualified).
    let def = quote!(::core::default::Default::default());

    // Adapt the original return into the full 3-tuple.
    let adapted = match original_output {
        ReturnType::Default => {
            // Allow "no return" as "all defaults" (optional; remove if you don't want this).
            quote! {
                (|| #original_block )();
                (#def, #def, #def)
            }
        }
        ReturnType::Type(_, ty_box) => {
            let ty: &Type = ty_box.as_ref();

            // ---- Single-value returns (no intents) ----

            // Output
            if is_output(ty) {
                quote! {
                    let output: <Self as ::block_traits::BlockSpecAssociatedTypes>::Output =
                        (|| #original_block )();
                    (output, #def, #def)
                }
            }
            // State
            else if is_state(ty) {
                quote! {
                    let state_out: <Self as ::block_traits::BlockSpecAssociatedTypes>::State =
                        (|| #original_block )();
                    (#def, state_out, #def)
                }
            }
            // Intents
            else if is_intents(ty) {
                quote! {
                    let intents: <Self as ::block_traits::BlockSpecAssociatedTypes>::Intents =
                        (|| #original_block )();
                    (#def, #def, intents)
                }
            }
            // ---- Tuple returns ----
            else if let Type::Tuple(tup) = ty {
                let elems: Vec<&Type> = tup.elems.iter().collect();

                // (Output, State)   (no intents)
                if elems.len() == 2 && is_output(elems[0]) && is_state(elems[1]) {
                    quote! {
                        let (output, state_out): (
                            <Self as ::block_traits::BlockSpecAssociatedTypes>::Output,
                            <Self as ::block_traits::BlockSpecAssociatedTypes>::State
                        ) = (|| #original_block )();
                        (output, state_out, #def)
                    }
                }
                // (Output, Intents)
                else if elems.len() == 2 && is_output(elems[0]) && is_intents(elems[1]) {
                    quote! {
                        let (output, intents): (
                            <Self as ::block_traits::BlockSpecAssociatedTypes>::Output,
                            <Self as ::block_traits::BlockSpecAssociatedTypes>::Intents
                        ) = (|| #original_block )();
                        (output, #def, intents)
                    }
                }
                // (State, Intents)
                else if elems.len() == 2 && is_state(elems[0]) && is_intents(elems[1]) {
                    quote! {
                        let (state_out, intents): (
                            <Self as ::block_traits::BlockSpecAssociatedTypes>::State,
                            <Self as ::block_traits::BlockSpecAssociatedTypes>::Intents
                        ) = (|| #original_block )();
                        (#def, state_out, intents)
                    }
                }
                // (Output, State, Intents)
                else if elems.len() == 3
                    && is_output(elems[0])
                    && is_state(elems[1])
                    && is_intents(elems[2])
                {
                    quote! {
                        let (output, state_out, intents): (
                            <Self as ::block_traits::BlockSpecAssociatedTypes>::Output,
                            <Self as ::block_traits::BlockSpecAssociatedTypes>::State,
                            <Self as ::block_traits::BlockSpecAssociatedTypes>::Intents
                        ) = (|| #original_block )();
                        (output, state_out, intents)
                    }
                } else {
                    return syn::Error::new(
                        tup.span(),
                        "unsupported return type for #[execute]. Allowed: Output, State, Intents, (Output, State), (Output, Intents), (State, Intents), (Output, State, Intents)",
                    )
                    .to_compile_error()
                    .into();
                }
            } else {
                return syn::Error::new(
                    ty.span(),
                    "unsupported return type for #[execute]. Allowed: Output, State, Intents, (Output, State), (Output, Intents), (State, Intents), (Output, State, Intents)",
                )
                .to_compile_error()
                .into();
            }
        }
    };

    // Replace body with the adapted one.
    *f.block = syn::parse_quote!({ #adapted });

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
