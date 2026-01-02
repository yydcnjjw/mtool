use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, format_ident, quote};
use syn::{
    Ident, LitInt, Path, Token, parenthesized,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token::{Comma, Paren},
};

pub struct EnumParams {
    count: usize,
    r#macro: Ident,
    idents: Punctuated<Ident, Token![,]>,
}

impl EnumParams {
    fn r#gen(&self) -> TokenStream2 {
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
            count,
            r#macro,
            idents: input.parse_terminated(Ident::parse)?,
        })
    }
}

impl ToTokens for EnumParams {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        tokens.extend([self.r#gen()]);
    }
}

pub struct EnumParamsWithIndex(EnumParams);

impl EnumParamsWithIndex {
    fn r#gen(&self) -> TokenStream2 {
        let EnumParams {
            count,
            r#macro,
            idents,
        } = &self.0;
        let params = (0..*count).map(|i| {
            let idents = idents.iter().map(|ident| format_ident!("{}{}", ident, i));
            quote! { ((#(#idents),*), #i) }
        });
        quote! { #r#macro!(#( #params ),*); }
    }
}

impl Parse for EnumParamsWithIndex {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self(input.parse()?))
    }
}

impl ToTokens for EnumParamsWithIndex {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        tokens.extend([self.r#gen()]);
    }
}
