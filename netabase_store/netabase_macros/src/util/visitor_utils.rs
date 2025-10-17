use syn::{Attribute, Fields, ItemStruct, Meta, PathSegment, Token, punctuated::Punctuated};

use crate::{
    errors::{LinkPathError, NetabaseModelDeriveError},
    item_info::netabase_model::{ModelKeyInfo, ModelLinkInfo},
    util::field_is_attribute,
    visitors::definitions_visitor::DefinitionsVisitor,
};

impl<'a> ModelKeyInfo<'a> {
    pub fn find_keys(fields: &'a Fields) -> Result<ModelKeyInfo<'a>, NetabaseModelDeriveError> {
        let primary_keys = match fields
            .iter()
            .find(|f| field_is_attribute(f, "primary_key").is_some())
        {
            Some(k) => k,
            None => return Err(NetabaseModelDeriveError::PrimaryKeyNotFound),
        };
        let secondary_keys = fields
            .iter()
            .filter(|f| field_is_attribute(f, "secondary_key").is_some())
            .collect();
        Ok(ModelKeyInfo {
            primary_keys,
            secondary_keys,
        })
    }
}

impl<'a> ModelLinkInfo<'a> {
    pub fn find_link(fields: &'a Fields) -> impl std::iter::Iterator<Item = ModelLinkInfo<'a>> {
        fields.iter().filter_map(|f| {
            if let Some(attribute) = field_is_attribute(f, "link") {
                let link_path = match Self::extract_path_from_metalist(attribute) {
                    Ok(r) => r,
                    Err(_) => return None,
                };
                Some(ModelLinkInfo {
                    link_path,
                    link_field: f,
                })
            } else {
                None
            }
        })
    }

    pub fn extract_path_from_metalist(
        attribute: &Attribute,
    ) -> Result<Punctuated<PathSegment, Token![::]>, NetabaseModelDeriveError> {
        if let Meta::List(meta_list) = &attribute.meta {
            match meta_list
                .parse_args_with(Punctuated::<PathSegment, Token![::]>::parse_terminated)
                .map_err(|e| e.into_compile_error())
            {
                Ok(r) => Ok(r),
                Err(e) => Err(NetabaseModelDeriveError::LinkPath(LinkPathError::Parse(e))),
            }
        } else {
            Err(NetabaseModelDeriveError::LinkPath(
                LinkPathError::IncorrectAttribute,
            ))
        }
    }
}

impl<'a> DefinitionsVisitor<'a> {
    pub fn check_derive(i: &ItemStruct, contains: &str) -> bool {
        i.attrs.iter().any(|att| {
            if att.path().is_ident("derive") {
                let mut result = false;
                att.parse_nested_meta(|meta| {
                    if meta.path.is_ident(contains) {
                        result = true;
                        return Ok(());
                    } else {
                        Err(meta.error("attribute is not derive"))
                    }
                });
                result
            } else {
                false
            }
        })
    }
}
