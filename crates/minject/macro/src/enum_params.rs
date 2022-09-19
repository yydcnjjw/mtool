use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token::Comma,
    Ident, LitInt, Token,
};

pub struct EnumParams {
    count: usize,
    r#macro: Ident,
    idents: Punctuated<Ident, Token![,]>,
}

impl EnumParams {
    fn gen(&self) -> TokenStream2 {
        let r#macro = &self.r#macro;
        let enum_idents = self.idents.iter().map(|ident| {
            let idents = (0..self.count).map(|v| format_ident!("{}{}", ident, v));
            quote! {#(#idents),*}
        });
        quote! { #r#macro!(#( #enum_idents ),*); }
    }
}

impl Parse for EnumParams {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let count = input.parse::<LitInt>()?.base10_parse()?;
        input.parse::<Comma>()?;

        let r#macro = input.parse::<Ident>()?;
        input.parse::<Comma>()?;

        Ok(Self {
            r#macro,
            count,
            idents: input.parse_terminated(Ident::parse)?,
        })
    }
}

impl ToTokens for EnumParams {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        tokens.extend([self.gen()]);
    }
}
