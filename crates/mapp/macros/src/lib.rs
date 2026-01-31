use proc_macro::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use syn::{DeriveInput, parse_macro_input, spanned::Spanned};

pub(crate) fn mapp_path() -> syn::Path {
    parse_str("::mapp")
}

/// Derive macro generating an impl of the trait `ScheduleLabel`.
///
/// This does not work for unions.
#[proc_macro_derive(ScheduleLabel)]
pub fn derive_schedule_label(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let mut trait_path = mapp_path();
    trait_path.segments.push(format_ident!("schedule").into());
    let mut dyn_eq_path = trait_path.clone();
    trait_path
        .segments
        .push(format_ident!("ScheduleLabel").into());
    dyn_eq_path.segments.push(format_ident!("DynEq").into());
    derive_label(input, "ScheduleLabel", &trait_path, &dyn_eq_path)
}

/// Derive macro generating an impl of the trait `TaskSet`.
///
/// This does not work for unions.
#[proc_macro_derive(TaskSet)]
pub fn derive_task_set(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let mut trait_path = mapp_path();
    trait_path.segments.push(format_ident!("schedule").into());
    let mut dyn_eq_path = trait_path.clone();
    trait_path.segments.push(format_ident!("TaskSet").into());
    dyn_eq_path.segments.push(format_ident!("DynEq").into());
    derive_label(input, "TaskSet", &trait_path, &dyn_eq_path)
}

/// Derive a label trait
///
/// # Args
///
/// - `input`: The [`syn::DeriveInput`] for struct that is deriving the label trait
/// - `trait_name`: Name of the label trait
/// - `trait_path`: The [path](`syn::Path`) to the label trait
/// - `dyn_eq_path`: The [path](`syn::Path`) to the `DynEq` trait
fn derive_label(
    input: syn::DeriveInput,
    trait_name: &str,
    trait_path: &syn::Path,
    dyn_eq_path: &syn::Path,
) -> TokenStream {
    if let syn::Data::Union(_) = &input.data {
        let message = format!("Cannot derive {trait_name} for unions.");
        return quote_spanned! {
            input.span() => compile_error!(#message);
        }
        .into();
    }

    let ident = input.ident.clone();
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let mut where_clause = where_clause.cloned().unwrap_or_else(|| syn::WhereClause {
        where_token: Default::default(),
        predicates: Default::default(),
    });
    where_clause.predicates.push(
        syn::parse2(quote! {
            Self: 'static + Send + Sync + Clone + Eq + ::core::fmt::Debug + ::core::hash::Hash
        })
        .unwrap(),
    );
    quote! {
        // To ensure alloc is available, but also prevent its name from clashing, we place the implementation inside an anonymous constant
        const _: () = {
            extern crate alloc;

            impl #impl_generics #trait_path for #ident #ty_generics #where_clause {
                fn dyn_clone(&self) -> alloc::boxed::Box<dyn #trait_path> {
                    alloc::boxed::Box::new(::core::clone::Clone::clone(self))
                }

                fn as_dyn_eq(&self) -> &dyn #dyn_eq_path {
                    self
                }

                fn dyn_hash(&self, mut state: &mut dyn ::core::hash::Hasher) {
                    let ty_id = ::core::any::TypeId::of::<Self>();
                    ::core::hash::Hash::hash(&ty_id, &mut state);
                    ::core::hash::Hash::hash(self, &mut state);
                }
            }
        };
    }
    .into()
}

/// Attempt to parse the provided [path](str) as a [syntax tree node](syn::parse::Parse)
fn try_parse_str<T: syn::parse::Parse>(path: &str) -> Option<T> {
    syn::parse(path.parse::<TokenStream>().ok()?).ok()
}

/// Attempt to parse provided [path](str) as a [syntax tree node](syn::parse::Parse).
///
/// # Panics
///
/// Will panic if the path is not able to be parsed. For a non-panicking option, see [`try_parse_str`]
///
/// [`try_parse_str`]: Self::try_parse_str
fn parse_str<T: syn::parse::Parse>(path: &str) -> T {
    try_parse_str(path).unwrap()
}
