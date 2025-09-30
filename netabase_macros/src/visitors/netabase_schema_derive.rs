use quote::ToTokens;
use syn::{Field, Ident, MetaList, parse_quote, visit::Visit, visit_mut::VisitMut};

use crate::append_ident;

#[derive(Default)]
pub struct NetabaseModelVisitor<'ast> {
    pub key_field: Option<&'ast Field>,
    pub key_derive: Option<&'ast MetaList>,
    pub name: Option<&'ast Ident>,
    pub key_name: Option<Ident>,
}

impl<'ast> Visit<'ast> for NetabaseModelVisitor<'ast> {
    fn visit_derive_input(&mut self, i: &'ast syn::DeriveInput) {
        self.name = Some(&i.ident);
        self.key_derive = i.attrs.iter().find_map(|att| {
            if att.path().is_ident("key_derive")
                && let syn::Meta::List(meta_list) = &att.meta
            {
                Some(meta_list)
            } else {
                None
            }
        });
        self.key_name = i.attrs.iter().find_map(|att| {
            if att.path().is_ident("key_name")
                && let syn::Meta::List(meta_list) = &att.meta
            {
                eprintln!("Visiting: {:?} with key: {:?}", self.name, self.key_name);
                let tok = &meta_list.tokens;
                Some(parse_quote! {#tok})
            } else {
                self.name.map(|name| append_ident(name, "Key"))
            }
        });
        if let syn::Data::Struct(data_struct) = &i.data
            && let Some(field) = data_struct
                .fields
                .iter()
                .find(|f| f.attrs.iter().any(|attr| attr.path().is_ident("key")))
        {
            self.key_field = Some(field);
        }
    }
}

/// A visitor that automatically adds `Encode` and `Decode` derives to struct definitions.
///
/// This visitor ensures that all structs have the necessary derive attributes for serialization,
/// either by adding them to existing `#[derive(...)]` attributes or creating new ones.
///
/// # Example Usage
/// ```rust
/// use syn::{parse_quote, visit_mut::VisitMut};
/// use crate::visitors::netabase_schema_derive::DeriveVisitor;
///
/// let mut ast: syn::DeriveInput = parse_quote! {
///     #[derive(Debug, Clone)]
///     struct MyStruct {
///         field: String,
///     }
/// };
///
/// let mut visitor = DeriveVisitor::new();
/// visitor.visit_derive_input_mut(&mut ast);
/// // Now ast has #[derive(Debug, Clone, Encode, Decode)]
/// ```
pub struct DeriveVisitor;

impl DeriveVisitor {
    /// Creates a new DeriveVisitor instance
    pub fn new() -> Self {
        Self
    }

    /// Checks if a derive attribute already contains the specified trait
    fn has_derive_trait(tokens: &proc_macro2::TokenStream, trait_name: &str) -> bool {
        tokens.to_string().contains(trait_name)
    }

    /// Adds missing Encode and Decode derives to an existing derive attribute
    fn add_missing_derives(&self, meta_list: &mut syn::MetaList) {
        use proc_macro2::TokenStream;
        use quote::quote;

        let has_encode = Self::has_derive_trait(&meta_list.tokens, "Encode");
        let has_decode = Self::has_derive_trait(&meta_list.tokens, "Decode");

        if !has_encode || !has_decode {
            let existing = &meta_list.tokens;
            let new_tokens: TokenStream = match (has_encode, has_decode) {
                (false, false) => quote! { #existing, Encode, Decode },
                (false, true) => quote! { #existing, Encode },
                (true, false) => quote! { #existing, Decode },
                (true, true) => return, // Both already present
            };

            meta_list.tokens = new_tokens;
        }
    }
}

impl VisitMut for DeriveVisitor {
    fn visit_derive_input_mut(&mut self, i: &mut syn::DeriveInput) {
        // Find existing derive attribute and add missing derives
        if let Some(derive_attr) = i
            .attrs
            .iter_mut()
            .find(|attr| attr.path().is_ident("derive"))
        {
            if let syn::Meta::List(ref mut meta_list) = derive_attr.meta {
                self.add_missing_derives(meta_list);
            }
        } else {
            // No derive attribute exists, create one with Encode and Decode
            i.attrs.push(parse_quote! { #[derive(Encode, Decode)] });
        }

        // Continue visiting child nodes
        syn::visit_mut::visit_derive_input_mut(self, i);
    }
}
