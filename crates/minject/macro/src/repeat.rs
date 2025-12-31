use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, quote};
use syn::{
    Expr, Ident, LitInt, Token, Type,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token::Comma,
};

pub struct Repeat {
    count: usize,
    r#macro: Type,
    rest: TokenStream2,
}

impl Repeat {
    fn r#gen(&self) -> TokenStream2 {
        let items = (0..self.count).// map(|v| v + 1).
            map(|v| {
            let r#macro = &self.r#macro;
            let rest = &self.rest;                
            quote! {
                #r#macro!(#v, #rest);
            }
        });
        quote! { #( #items )* }
    }
}

impl Parse for Repeat {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let count = input.parse::<LitInt>()?.base10_parse()?;
        input.parse::<Comma>()?;

        let r#macro = input.parse::<Type>()?;
        input.parse::<Comma>()?;

        Ok(Self {
            count,
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
