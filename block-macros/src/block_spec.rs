use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Attribute, Item, ItemMod};

pub fn block_spec_impl(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemMod);

    // Extract the module content
    let (_brace, items) = match input.content {
        Some(content) => content,
        None => return quote! { #input }.into(),
    };

    // Find the structs with our attributes
    let mut input_structs = Vec::new();
    let mut output_structs = Vec::new();
    let mut state_structs = Vec::new();
    let mut block_structs = Vec::new();

    for item in &items {
        if let Item::Struct(item_struct) = item {
            if has_attribute(&item_struct.attrs, "input") {
                input_structs.push(&item_struct.ident);
            } else if has_attribute(&item_struct.attrs, "output") {
                output_structs.push(&item_struct.ident);
            } else if has_attribute(&item_struct.attrs, "state") {
                state_structs.push(&item_struct.ident);
            } else if has_attribute(&item_struct.attrs, "block") {
                block_structs.push(&item_struct.ident);
            }
        }
    }

    // Validate we have exactly one of each
    let validation_errors = check_struct_counts(
        &input_structs,
        &output_structs,
        &state_structs,
        &block_structs,
    );

    if !validation_errors.is_empty() {
        let error_msg = validation_errors.join("; ");
        return syn::Error::new_spanned(&input.ident, error_msg)
            .to_compile_error()
            .into();
    }

    // We know we have exactly one of each now
    let input_struct = input_structs[0];
    let output_struct = output_structs[0];
    let state_struct = state_structs[0];
    let block_struct = block_structs[0];

    let input_keys = syn::Ident::new(&format!("{}Keys", input_struct), input_struct.span());
    let output_keys = syn::Ident::new(&format!("{}Keys", output_struct), output_struct.span());

    let block_impl = quote! {
        pub type InputKeys = #input_keys;
        pub type OutputKeys = #output_keys;

        impl blocks::BlockSpec for #block_struct {
            type Input = #input_struct;
            type Output = #output_struct;
            type State = #state_struct;

            type InputKeys = #input_keys;
            type OutputKeys = #output_keys;

            fn init_state(&self) -> Self::State {
                self.init_state()
            }

            fn execute(&self, input: Self::Input, state: &Self::State) -> (Self::Output, Self::State) {
                self.execute(input, state)
            }

            fn register_outputs(&self, registry: &mut registry::Registry, out_keys: &Self::OutputKeys) {
                <Self::OutputKeys as registry::OutputKeys<Self::Output>>::register(out_keys, registry)
            }
        }
    };

    let mod_ident = &input.ident;
    let mod_vis = &input.vis;
    let mod_attrs = &input.attrs;

    quote! {
        #(#mod_attrs)*
        #mod_vis mod #mod_ident {
            #(#items)*

            #block_impl
        }
    }
    .into()
}

fn check_struct_counts(
    input_structs: &[&syn::Ident],
    output_structs: &[&syn::Ident],
    state_structs: &[&syn::Ident],
    block_structs: &[&syn::Ident],
) -> Vec<String> {
    let mut errors = Vec::new();

    match input_structs.len() {
        0 => errors.push("Missing #[input] struct".to_string()),
        1 => {}
        n => errors.push(format!("Found {} #[input] structs, expected exactly 1", n)),
    }

    match output_structs.len() {
        0 => errors.push("Missing #[output] struct".to_string()),
        1 => {}
        n => errors.push(format!("Found {} #[output] structs, expected exactly 1", n)),
    }

    match state_structs.len() {
        0 => errors.push("Missing #[state] struct".to_string()),
        1 => {}
        n => errors.push(format!("Found {} #[state] structs, expected exactly 1", n)),
    }

    match block_structs.len() {
        0 => errors.push("Missing #[block] struct".to_string()),
        1 => {}
        n => errors.push(format!("Found {} #[block] structs, expected exactly 1", n)),
    }

    errors
}

fn has_attribute(attrs: &[Attribute], name: &str) -> bool {
    attrs.iter().any(|attr| attr.path().is_ident(name))
}
