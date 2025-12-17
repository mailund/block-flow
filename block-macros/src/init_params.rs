use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemStruct};

pub fn init_params_impl(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let s = parse_macro_input!(item as ItemStruct);

    let expanded = quote! {
        #[::serialization_macros::serializable_struct]
        #s
    };

    expanded.into()
}
