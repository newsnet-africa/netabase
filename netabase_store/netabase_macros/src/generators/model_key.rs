use syn::{Ident, ItemEnum, ItemStruct, parse_quote};

use crate::{util::append_ident, visitors::model_visitor::ModelVisitor};

impl<'a> ModelVisitor<'a> {
    pub fn generate_keys(&self) -> (ItemStruct, Vec<ItemStruct>, ItemEnum, ItemEnum) {
        let p_keys = match Self::generate_primary_key(&self) {
            Ok(k) => k,
            Err(e) => panic!("{}", e),
        };
        let primary_key_id = p_keys.ident.clone();
        let secondary_newtypes = self.generate_secondary_keys_newtypes();
        let secondary_keys = self.generate_secondary_keys(&secondary_newtypes);
        let secondary_newtypes = secondary_newtypes.iter().map(|(s, _)| s.clone()).collect();
        let secondary_key_id = secondary_keys.ident.clone();
        let name = match self.name {
            Some(n) => append_ident(n, "Key"),
            None => panic!("Visitor error (parsing struct name?)"),
        };
        (
            p_keys,
            secondary_newtypes,
            secondary_keys,
            parse_quote!(
                #[derive(Debug, Clone, ::netabase_store::strum::EnumDiscriminants,
                    ::netabase_store::derive_more::From, ::netabase_store::derive_more::TryInto,
                    ::netabase_store::bincode::Encode, ::netabase_store::bincode::Decode
                )]
                pub enum #name {
                    Primary(#primary_key_id),
                    Secondary(#secondary_key_id),
                }
            ),
        )
    }

    pub fn generate_model_trait_impl(&self) -> proc_macro2::TokenStream {
        let model_name = match self.name {
            Some(n) => n,
            None => panic!("Visitor error (parsing struct name?)"),
        };
        let primary_key_ty = match Self::generate_primary_key(&self) {
            Ok(k) => k.ident,
            Err(e) => panic!("{}", e),
        };
        let secondary_keys_ty = append_ident(model_name, "SecondaryKeys");
        let keys_ty = append_ident(model_name, "Key");

        // Get the primary key field identifier
        let primary_field = match &self.key {
            Some(k) => k.primary_keys.ident.as_ref().unwrap(),
            None => panic!("Primary key not found"),
        };

        // Get secondary key field identifiers
        let secondary_fields: Vec<_> = match &self.key {
            Some(k) => k
                .secondary_keys
                .iter()
                .map(|f| {
                    let field_name = f.ident.as_ref().unwrap();
                    let field_name_upper = heck::AsUpperCamelCase(field_name.to_string());
                    let variant = Ident::new(
                        &field_name_upper.to_string(),
                        proc_macro2::Span::call_site(),
                    );
                    quote::quote! {
                        #secondary_keys_ty::#variant(self.#field_name.clone().into())
                    }
                })
                .collect(),
            None => vec![],
        };

        let discriminant_name = model_name.to_string();

        quote::quote! {
            impl ::netabase_store::traits::model::NetabaseModelTrait for #model_name {
                type PrimaryKey = #primary_key_ty;
                type SecondaryKeys = #secondary_keys_ty;
                type Keys = #keys_ty;

                fn primary_key(&self) -> Self::PrimaryKey {
                    #primary_key_ty(self.#primary_field.clone())
                }

                fn secondary_keys(&self) -> Vec<Self::SecondaryKeys> {
                    vec![#(#secondary_fields),*]
                }

                fn discriminant_name() -> &'static str {
                    #discriminant_name
                }
            }

            impl ::netabase_store::traits::model::NetabaseModelTraitKey for #primary_key_ty {}
            impl ::netabase_store::traits::model::NetabaseModelTraitKey for #secondary_keys_ty {}
            impl ::netabase_store::traits::model::NetabaseModelTraitKey for #keys_ty {}
        }
    }
}

mod key_gen {
    use syn::{Field, Ident, ItemEnum, ItemStruct, Variant, parse_quote};

    use crate::{
        errors::NetabaseModelDeriveError, util::append_ident, visitors::model_visitor::ModelVisitor,
    };

    impl<'a> ModelVisitor<'a> {
        fn generate_newtype<'ast>(field: &'ast Field, name: &Ident) -> ItemStruct {
            let ty = &field.ty;
            parse_quote!(
                #[derive(Debug, Clone, PartialEq, Eq, ::netabase_store::derive_more::From, ::netabase_store::derive_more::Into,
                    ::netabase_store::bincode::Encode, ::netabase_store::bincode::Decode
                )]
                pub struct #name(pub #ty);
            )
        }

        pub fn generate_primary_key(&self) -> Result<ItemStruct, NetabaseModelDeriveError> {
            let key = match &self.key {
                Some(k) => k,
                None => return Err(NetabaseModelDeriveError::PrimaryKeyNotFound),
            };
            let name = match &self.name {
                Some(n) => n,
                None => return Err(NetabaseModelDeriveError::MacroVisitorError),
            };
            Ok(Self::generate_newtype(
                key.primary_keys,
                &append_ident(name, "PrimaryKey"),
            ))
        }

        pub fn generate_secondary_keys_newtypes(&self) -> Vec<(ItemStruct, Ident)> {
            let key = match &self.key {
                Some(k) => k,
                None => return vec![],
            };

            key.secondary_keys
                .iter()
                .map(|f| {
                    let ident = if let Some(id) = &f.ident {
                        let id = heck::AsUpperCamelCase(id.to_string());
                        Ident::new(&id.to_string(), proc_macro2::Span::call_site())
                    } else {
                        panic!("Struct fields must be named")
                    };
                    (
                        Self::generate_newtype(f, &append_ident(&ident, "SecondaryKey")),
                        ident,
                    )
                })
                .collect()
        }

        pub fn generate_variant(keys: &(ItemStruct, Ident)) -> Variant {
            let name = &keys.1;
            let ty = &keys.0.ident;
            parse_quote! {
                #name(#ty)
            }
        }

        pub fn generate_secondary_keys(&self, keys: &Vec<(ItemStruct, Ident)>) -> ItemEnum {
            let list = keys.iter().map(|f| Self::generate_variant(f));
            let name = match &self.name {
                Some(n) => &append_ident(n, "SecondaryKeys"),
                None => panic!("Visitor not initialised"),
            };
            parse_quote!(
                #[derive(Debug, Clone, ::netabase_store::strum::EnumDiscriminants,
                    ::netabase_store::derive_more::From, ::netabase_store::derive_more::TryInto,
                    ::netabase_store::bincode::Encode, ::netabase_store::bincode::Decode
                )]
                pub enum #name {
                    #(#list),*
                }
            )
        }
    }
}
