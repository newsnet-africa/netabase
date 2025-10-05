use syn::{ItemImpl, ItemType, parse_quote};

use crate::{
    NetabaseModelVisitor, append_ident,
    util::{netabase_model_trait_path, relational_link_type_path},
};

impl<'ast> NetabaseModelVisitor<'ast> {
    pub fn generate_netabase_model_trait(&self) -> ItemImpl {
        let name = match self.name {
            Some(id) => id,
            None => panic!("Cannot find model name"),
        };

        let key_field = match self.key_field {
            Some(id) => match &id.ident {
                Some(id) => id,
                None => panic!("Ident not found"),
            },
            None => panic!("Cannot find model key"),
        };

        let key_name = match &self.key_name {
            Some(k) => k,
            None => {
                // Generate default key name if not provided
                match self.name {
                    Some(name) => {
                        eprintln!("Generating default key name for: {:?}", name);
                        &append_ident(name, "Key")
                    }
                    None => {
                        eprintln!("No model name available, using placeholder");
                        &syn::Ident::new("PlaceholderKey", proc_macro2::Span::call_site())
                    }
                }
            }
        };

        let _secondary_keys_name = append_ident(name, "SecondaryKeys");
        let _relations_name = append_ident(name, "Relations");

        // Generate secondary keys method
        let secondary_key_names: Vec<proc_macro2::TokenStream> = self
            .secondary_key_fields
            .iter()
            .filter_map(|field| {
                field.ident.as_ref().map(|ident| {
                    let ident_str = ident.to_string();
                    quote::quote! { #ident_str }
                })
            })
            .collect();

        let secondary_keys_method: syn::ImplItemFn = parse_quote! {
            fn secondary_keys() -> Vec<&'static str> {
                vec![#(#secondary_key_names),*]
            }
        };

        // Generate relations method (always empty now)
        let relations_method: syn::ImplItemFn = parse_quote! {
            fn relations() -> Vec<&'static str> {
                vec![]
            }
        };

        let relations_name = append_ident(name, "Relations");

        // Generate discriminant type name for relations (always empty now)
        let relations_discriminants = relations_name.clone();
        let trait_path = netabase_model_trait_path();

        parse_quote! {
            impl #trait_path for #name {
                type Key = #key_name;
                type RelationsDiscriminants = #relations_discriminants;

                fn key(&self) -> Self::Key {
                    self.#key_field.clone().into()
                }

                fn tree_name() -> &'static str {
                    stringify!(#name)
                }

                #secondary_keys_method

                #relations_method

                fn relation_discriminants() -> Vec<Self::RelationsDiscriminants> {
                    <Self::RelationsDiscriminants as ::netabase_deps::__private::strum::IntoEnumIterator>::iter().collect()
                }
            }
        }
    }

    pub fn generate_type_alias(&self) -> ItemType {
        let name = match self.name {
            Some(id) => id,
            None => panic!("Cannot find model name"),
        };

        let key_name = match &self.key_name {
            Some(k) => k,
            None => &append_ident(name, "Key"),
        };

        let alias_name = append_ident(name, "Link");
        let relational_link_path = relational_link_type_path();

        parse_quote! {
            pub type #alias_name = #relational_link_path<#key_name, #name>;
        }
    }
}

pub mod key_impl {
    use syn::{DeriveInput, ItemImpl, parse_quote};

    pub fn generate_netabase_model_key_trait(key_struct: &DeriveInput) -> ItemImpl {
        let name = &key_struct.ident;
        let trait_path = crate::util::netabase_model_key_trait_path();
        parse_quote! {
            impl #trait_path for #name {

            }
        }
    }
}
