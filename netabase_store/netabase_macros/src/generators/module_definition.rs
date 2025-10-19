use syn::{Ident, ItemEnum};

use crate::{
    generators::module_definition::def_gen::generate_enums, util::append_ident,
    visitors::definitions_visitor::DefinitionsVisitor,
};

impl<'a> DefinitionsVisitor<'a> {
    pub fn generate_definitions(
        &self,
        definition: &Ident,
        definition_key: &Ident,
    ) -> (ItemEnum, ItemEnum) {
        generate_enums(&self.modules, definition, definition_key)
    }

    pub fn generate_definition_trait_impls(
        &self,
        definition: &Ident,
        definition_key: &Ident,
    ) -> proc_macro2::TokenStream {
        let discriminants = append_ident(definition, "Discriminants");
        let key_discriminants = append_ident(definition_key, "Discriminants");

        quote::quote! {
            impl ::netabase_store::traits::definition::NetabaseDefinitionTrait for #definition {
                type Discriminants = #discriminants;
                type Keys = #definition_key;

                fn discriminant(&self) -> Self::Discriminants {
                    // EnumDiscriminants automatically generates From<T> for TDiscriminants
                    self.clone().into()
                }
            }

            impl ::netabase_store::traits::definition::NetabaseDefinitionTraitKey for #definition_key {
                type Discriminants = #key_discriminants;
                type Definition = #definition;

                fn discriminant(&self) -> Self::Discriminants {
                    self.clone().into()
                }
            }

            impl ::netabase_store::traits::convert::ToIVec for #definition {}
            impl ::netabase_store::traits::convert::ToIVec for #definition_key {}

            impl ::std::convert::From<#discriminants> for String {
                fn from(d: #discriminants) -> String {
                    format!("{:?}", d)
                }
            }

            impl ::std::convert::From<#key_discriminants> for String {
                fn from(d: #key_discriminants) -> String {
                    format!("{:?}", d)
                }
            }

            impl ::std::fmt::Display for #discriminants {
                fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                    write!(f, "{:?}", self)
                }
            }

            impl ::std::fmt::Display for #key_discriminants {
                fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                    write!(f, "{:?}", self)
                }
            }
        }
    }
}

pub mod def_gen {
    use syn::{
        Ident, ItemEnum, ItemStruct, PathSegment, Token, Variant, parse_quote,
        punctuated::Punctuated,
    };

    use crate::{item_info::netabase_definitions::ModuleInfo, util::append_ident};

    pub fn generate_model_variants(
        structs: &Vec<&ItemStruct>,
        path: Punctuated<PathSegment, Token![::]>,
    ) -> Vec<Variant> {
        structs
            .iter()
            .map(|s| {
                let name = &s.ident;
                let mut inner_path = path.clone();
                inner_path.push(name.clone().into());
                parse_quote! {
                    #name(#inner_path)
                }
            })
            .collect()
    }
    pub fn generate_model_key_variants(
        keys: &Vec<Ident>,
        path: Punctuated<PathSegment, Token![::]>,
    ) -> Vec<Variant> {
        keys.iter()
            .map(|name| {
                let mut inner_path = path.clone();
                inner_path.push(name.clone().into());
                parse_quote! {
                    #name(#inner_path)
                }
            })
            .collect()
    }
    pub fn generate_enums(
        modules: &Vec<ModuleInfo<'_>>,
        definition: &Ident,
        definition_key: &Ident,
    ) -> (ItemEnum, ItemEnum) {
        let models = modules
            .iter()
            .flat_map(|m| generate_model_variants(&m.models, m.path.clone()));
        let keys = modules
            .iter()
            .flat_map(|m| generate_model_key_variants(&m.keys, m.path.clone()));

        (
            parse_quote! {
                #[derive(Debug, Clone, ::netabase_store::netabase_deps::strum::IntoStaticStr, ::netabase_store::netabase_deps::strum::EnumDiscriminants,
                    ::netabase_store::netabase_deps::derive_more::From,::netabase_store::netabase_deps::derive_more::TryInto,
                    ::netabase_store::netabase_deps::bincode::Encode, ::netabase_store::netabase_deps::bincode::Decode
                )]
                #[strum_discriminants(derive(Hash, ::netabase_store::netabase_deps::strum::EnumIter))]
                pub enum #definition {
                    #(#models),*
                }
            },
            parse_quote! {
                #[derive(Debug, Clone, ::netabase_store::netabase_deps::strum::EnumDiscriminants,
                    ::netabase_store::netabase_deps::derive_more::From, ::netabase_store::netabase_deps::derive_more::TryInto,
                    ::netabase_store::netabase_deps::bincode::Encode, ::netabase_store::netabase_deps::bincode::Decode
                )]
                #[strum_discriminants(derive(Hash))]
                pub enum #definition_key {
                    #(#keys),*
                }
            },
        )
    }
}
