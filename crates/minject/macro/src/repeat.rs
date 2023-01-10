use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token::Comma,
    Ident, LitInt, Token,
};

pub struct Repeat {
    count: usize,
    r#macro: Ident,
    rest_args: Punctuated<Ident, Token![,]>,
}

impl Repeat {
    fn gen(&self) -> TokenStream2 {
        let items = (0..self.count).// map(|v| v + 1).
            map(|v| {
            let (r#macro, rest_args) = (&self.r#macro, self.rest_args.iter());
            quote! {
                #r#macro!(#v, #( #rest_args ),*);
            }
        });
        quote! { #( #items )* }
    }
}

impl Parse for Repeat {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let count = input.parse::<LitInt>()?.base10_parse()?;
        input.parse::<Comma>()?;

        let r#macro = input.parse::<Ident>()?;
        input.parse::<Comma>()?;

        Ok(Self {
            count,
            r#macro,
            rest_args: input.parse_terminated(Ident::parse)?,
        })
    }
}

impl ToTokens for Repeat {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        tokens.extend([self.gen()]);
    }
}

