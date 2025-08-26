mod generators;
mod visitors;

extern crate proc_macro;
use proc_macro::TokenStream;

use quote::{ToTokens, quote};
use syn::{DeriveInput, parse_macro_input, visit::Visit};

use crate::{
    generate_netabase_impl::{generate_netabase_impl, generate_netabase_macro},
    generators::generate_netabase_impl,
    visitors::SchemaValidator,
};

/// Example of [function-like procedural macro][1].
///
/// [1]: https://doc.rust-lang.org/reference/procedural-macros.html#function-like-procedural-macros
#[proc_macro]
pub fn my_macro(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let tokens = quote! {
        #input

        struct Hello;
    };

    tokens.into()
}

/// Example of user-defined [derive mode macro][1]
///
/// [1]: https://doc.rust-lang.org/reference/procedural-macros.html#derive-mode-macros
#[proc_macro_derive(NetabaseSchema, attributes(key))]
pub fn my_derive(input: TokenStream) -> TokenStream {
    let inp = parse_macro_input!(input as DeriveInput);
    let mut vi = SchemaValidator::default();
    vi.visit_derive_input(&inp);
    let net_impl = generate_netabase_macro(vi);

    quote! {
        #net_impl
    }
    .into()
}

/// Example of user-defined [procedural macro attribute][1].
///
/// [1]: https://doc.rust-lang.org/reference/procedural-macros.html#attribute-macros
#[proc_macro_attribute]
pub fn my_attribute(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let tokens = quote! {
        #input

        struct Hello;
    };

    tokens.into()
}
