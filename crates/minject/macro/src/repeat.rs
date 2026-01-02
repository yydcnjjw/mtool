use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, quote};
use syn::{
    LitInt, Type,
    parse::{Parse, ParseStream},
    token::Comma,
};

pub struct Repeat {
    start: usize,
    end: usize,
    r#macro: Type,
    rest: TokenStream2,
}

impl Repeat {
    fn r#gen(&self) -> TokenStream2 {
        let items = (self.start..self.end).map(|i| {
            let r#macro = &self.r#macro;
            let rest = &self.rest;
            quote! {
                #r#macro!(#i, #rest);
            }
        });
        quote! { #( #items )* }
    }
}

impl Parse for Repeat {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let start = input.parse::<LitInt>()?.base10_parse()?;
        input.parse::<Comma>()?;

        let end = input.parse::<LitInt>()?.base10_parse()?;
        input.parse::<Comma>()?;

        let r#macro = input.parse::<Type>()?;
        input.parse::<Comma>()?;

        Ok(Self {
            start,
            end,
            r#macro,
            rest: input.parse()?,
        })
    }
}

impl ToTokens for Repeat {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        tokens.extend([self.r#gen()]);
    }
}
