use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token::Comma,
    ImplItemMethod
};

pub struct Inject {
    method: ImplItemMethod,
}

impl Inject {
    fn gen(&self) -> TokenStream2 {
        quote!()
    }
}

impl Parse for Inject {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            method: input.parse()?,
        })
    }
}

impl ToTokens for Inject {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        tokens.extend([self.gen()]);
    }
}
