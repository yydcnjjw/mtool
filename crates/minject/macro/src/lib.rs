mod enum_params;
mod repeat;
// mod inject;

use enum_params::EnumParams;
// use inject::Inject;
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


// #[proc_macro]
// pub fn inject(input: TokenStream) -> TokenStream {
//     parse_macro_input!(input as Inject)
//         .into_token_stream()
//         .into()
// }
