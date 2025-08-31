mod enum_params;
mod repeat;

use enum_params::{EnumParams, EnumParamsWithIndex};
use proc_macro::TokenStream;
use quote::ToTokens;
use repeat::Repeat;
use syn::parse_macro_input;

#[proc_macro]
pub fn repeat(input: TokenStream) -> TokenStream {
    parse_macro_input!(input as Repeat)
        .into_token_stream()
        .into()
}

#[proc_macro]
pub fn enum_params(input: TokenStream) -> TokenStream {
    parse_macro_input!(input as EnumParams)
        .into_token_stream()
        .into()
}

#[proc_macro]
pub fn enum_params_with_index(input: TokenStream) -> TokenStream {
    parse_macro_input!(input as EnumParamsWithIndex)
        .into_token_stream()
        .into()
}

