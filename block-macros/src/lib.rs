use proc_macro::TokenStream;

mod block;
mod init_params;
mod input;
mod output;

#[proc_macro_attribute]
pub fn input(attr: TokenStream, item: TokenStream) -> TokenStream {
    input::input_impl(attr, item)
}

#[proc_macro_attribute]
pub fn output(attr: TokenStream, item: TokenStream) -> TokenStream {
    output::output_impl(attr, item)
}

#[proc_macro_attribute]
pub fn state(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Just a marker for now - return input unchanged
    item
}

#[proc_macro_attribute]
pub fn block(attr: TokenStream, item: TokenStream) -> TokenStream {
    block::block_impl(attr, item)
}

#[proc_macro_attribute]
pub fn init_params(attr: TokenStream, item: TokenStream) -> TokenStream {
    init_params::init_params_impl(attr, item)
}

//use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Ident, Result, Token,
};

/// make_defaults!(input, output, init_params, state)
/// make_defaults!(input=MyInput, state=MyState)
#[proc_macro]
pub fn make_defaults(input: TokenStream) -> TokenStream {
    let spec = syn::parse_macro_input!(input as Spec);

    // Track which ones were requested, and with what names.
    let mut want_input: Option<Ident> = None;
    let mut want_output: Option<Ident> = None;
    let mut want_init_param: Option<Ident> = None;
    let mut want_state: Option<Ident> = None;

    for item in &spec.items {
        match item.kind.to_string().as_str() {
            "input" => set_once(&mut want_input, item.name.clone(), "input"),
            "output" => set_once(&mut want_output, item.name.clone(), "output"),
            "init_params" => set_once(&mut want_init_param, item.name.clone(), "init_params"),
            "state" => set_once(&mut want_state, item.name.clone(), "state"),
            other => {
                return syn::Error::new(
                    item.kind.span(),
                    format!(
                        "unknown kind `{other}`; expected one of: input, output, init_params, state"
                    ),
                )
                .to_compile_error()
                .into();
            }
        }
    }

    // Provide default names if requested but not explicitly renamed.
    let input_name = want_input.or(None);
    let output_name = want_output.or(None);
    let init_param_name = want_init_param.or(None);
    let state_name = want_state.or(None);

    let mut out = proc_macro2::TokenStream::new();

    if let Some(name) = input_name.or_else(|| spec.default_name("input")) {
        out.extend(gen_struct("input", &name));
    }
    if let Some(name) = output_name.or_else(|| spec.default_name("output")) {
        out.extend(gen_struct("output", &name));
    }
    if let Some(name) = init_param_name.or_else(|| spec.default_name("init_params")) {
        out.extend(gen_struct("init_params", &name));
    }
    if let Some(name) = state_name.or_else(|| spec.default_name("state")) {
        out.extend(gen_struct("state", &name));
    }

    out.into()
}

/// Generates:
/// #[<attr>]
/// pub struct <Name>;
fn gen_struct(attr_name: &str, name: &Ident) -> proc_macro2::TokenStream {
    let attr_ident = Ident::new(attr_name, Span::call_site());
    quote! {
        #[#attr_ident]
        pub struct #name;
    }
}

/// Ensure each kind is only specified once.
fn set_once(slot: &mut Option<Ident>, new: Option<Ident>, kind: &str) {
    if slot.is_some() {
        // We can’t return Result easily from here; caller validates via early compile error elsewhere
        // by simply overwriting, but better to panic with a clear message. (Panics become compile errors.)
        panic!("`{kind}` specified more than once");
    }
    *slot = Some(new.unwrap_or_else(|| default_ident_for(kind)));
}

fn default_ident_for(kind: &str) -> Ident {
    match kind {
        "input" => Ident::new("Input", Span::call_site()),
        "output" => Ident::new("Output", Span::call_site()),
        "init_params" => Ident::new("InitParams", Span::call_site()),
        "state" => Ident::new("State", Span::call_site()),
        _ => Ident::new("Unknown", Span::call_site()),
    }
}

/// Parsed macro input: a comma-separated list of `kind` or `kind=Name`
struct Spec {
    items: Vec<SpecItem>,
}

impl Spec {
    /// If the user *didn't request* a kind, don't generate it.
    /// If they did request it without a rename, `set_once` already filled a default ident.
    fn default_name(&self, kind: &str) -> Option<Ident> {
        // Only generate kinds that appear in input.
        let requested = self.items.iter().any(|it| it.kind == kind);
        if !requested {
            return None;
        }
        // If requested but name was empty (no rename), set_once already placed a default,
        // but this function is used for completeness.
        Some(default_ident_for(kind))
    }
}

struct SpecItem {
    kind: Ident,
    name: Option<Ident>,
}

impl Parse for Spec {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.is_empty() {
            return Ok(Spec { items: vec![] });
        }

        let punct: Punctuated<SpecItem, Token![,]> = Punctuated::parse_terminated(input)?;

        Ok(Spec {
            items: punct.into_iter().collect(),
        })
    }
}

impl Parse for SpecItem {
    fn parse(input: ParseStream) -> Result<Self> {
        let kind: Ident = input.parse()?;
        let name = if input.peek(Token![=]) {
            let _eq: Token![=] = input.parse()?;
            Some(input.parse()?)
        } else {
            None
        };

        Ok(SpecItem { kind, name })
    }
}
