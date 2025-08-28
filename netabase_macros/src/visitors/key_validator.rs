use crate::SchemaEnumGenerator;
use crate::visitors::Key;
use crate::visitors::VisitError;
use crate::visitors::validation_error::{InnerKeyError, KeyError};

use syn::FnArg;
use syn::ReturnType;
use syn::{Data, DeriveInput, Expr, Field, Fields, Meta, Signature, Variant};

pub(super) fn find_outer_key_fn_path(item: &DeriveInput) -> Result<Key<'_>, VisitError> {
    let mut found_keys: Vec<Signature> = vec![];
    item.attrs.iter().for_each(|att| {
        if let Meta::NameValue(name_value) = &att.meta
            && att.path().is_ident("key")
            && let Expr::Lit(expr_lit) = &name_value.value
            && let syn::Lit::Str(lit_str) = &expr_lit.lit
        {
            let sig: Signature = lit_str.parse().expect("Parse Erruh");
            if let Some(FnArg::Receiver(rec)) = sig.inputs.first()
                && sig.inputs.iter().count().eq(&1)
                && let ReturnType::Type(_, _) = sig.output
            {
                found_keys.push(sig);
            }
            // else if ReturnType::Default == sig.output {
            //     panic!("Function needs a Return type")
            // }
            else {
                panic!("Function should only contain `&self` receiver as argument")
            }
        }
    });
    let key_count = found_keys.len();
    if key_count == 1
        && let Some(path) = found_keys.first()
    {
        Ok(Key::Outer {
            sig: Box::new(path.clone()),
        })
    } else if key_count == 0 {
        Err(VisitError::KeyError(KeyError::InnerKeyError(
            InnerKeyError::InnerKeyNotFound,
        )))
    } else {
        Err(VisitError::KeyError(KeyError::TooManyKeys))
    }
}

pub(super) fn find_inner_key<'ast>(item: &'ast Data) -> Result<Key<'ast>, VisitError> {
    match item {
        Data::Struct(struct_data) => match &struct_data.fields {
            Fields::Named(named_fields) => {
                let mut found_keys: Vec<&Field> = vec![];
                for field in named_fields.named.iter() {
                    if field.attrs.iter().any(|att: &syn::Attribute| {
                        if let Meta::Path(path) = &att.meta
                            && path
                                .segments
                                .iter()
                                .any(|seg| seg.ident.to_string().eq(&"key".to_string()))
                        {
                            true
                        } else {
                            false
                        }
                    }) {
                        found_keys.push(field);
                    }
                }
                let key_count = found_keys.len();
                if key_count == 1
                    && let Some(path) = found_keys.first()
                {
                    Ok(Key::StructInner { field: path })
                } else if key_count == 0 {
                    Err(VisitError::KeyError(KeyError::InnerKeyError(
                        InnerKeyError::InnerKeyNotFound,
                    )))
                } else {
                    Err(VisitError::KeyError(KeyError::TooManyKeys))
                }
            }
            Fields::Unnamed(unnamed_fields) => {
                let mut found_keys: Vec<&Field> = vec![];
                if let Some(field) = unnamed_fields.unnamed.first() {
                    if field.attrs.iter().any(|att| {
                        if let Meta::Path(path) = &att.meta
                            && path
                                .segments
                                .iter()
                                .any(|seg| seg.ident.to_string().eq(&"key".to_string()))
                        {
                            true
                        } else {
                            false
                        }
                    }) {
                        found_keys.push(field);
                    }
                } else {
                    return Err(VisitError::KeyError(KeyError::InnerKeyError(
                        InnerKeyError::KeyNotFirstTupleItem,
                    )));
                }
                let key_count = found_keys.len();
                if key_count == 1
                    && let Some(path) = found_keys.first()
                {
                    Ok(Key::StructInner { field: path })
                } else if key_count == 0 {
                    Err(VisitError::KeyError(KeyError::InnerKeyError(
                        InnerKeyError::InnerKeyNotFound,
                    )))
                } else {
                    Err(VisitError::KeyError(KeyError::TooManyKeys))
                }
            }
            Fields::Unit => Err(VisitError::InvalidSchemaType),
        },
        Data::Enum(enum_data) => {
            let mut var_keys: Vec<(&'ast Variant, &'ast Field)> = vec![];
            for var in &enum_data.variants {
                match &var.fields {
                    Fields::Named(named_fields) => {
                        let mut found_keys: Vec<&Field> = vec![];
                        for field in named_fields.named.iter() {
                            if field.attrs.iter().any(|att: &syn::Attribute| {
                                if let Meta::Path(path) = &att.meta
                                    && path
                                        .segments
                                        .iter()
                                        .any(|seg| seg.ident.to_string().eq(&"key".to_string()))
                                {
                                    true
                                } else {
                                    false
                                }
                            }) {
                                found_keys.push(field);
                            }
                        }
                        let key_count = found_keys.len();
                        if key_count == 1
                            && let Some(path) = found_keys.first()
                        {
                            var_keys.push((var, path));
                        } else if key_count == 0 {
                            return Err(VisitError::KeyError(KeyError::InnerKeyError(
                                InnerKeyError::InnerKeyNotFound,
                            )));
                        } else {
                            return Err(VisitError::KeyError(KeyError::TooManyKeys));
                        }
                    }
                    Fields::Unnamed(unnamed_fields) => {
                        let mut found_keys: Vec<&Field> = vec![];
                        if let Some(field) = unnamed_fields.unnamed.first() {
                            if field.attrs.iter().any(|att| {
                                if let Meta::Path(path) = &att.meta
                                    && path
                                        .segments
                                        .iter()
                                        .any(|seg| seg.ident.to_string().eq(&"key".to_string()))
                                {
                                    true
                                } else {
                                    false
                                }
                            }) {
                                found_keys.push(field);
                            }
                        } else {
                            return Err(VisitError::KeyError(KeyError::InnerKeyError(
                                InnerKeyError::KeyNotFirstTupleItem,
                            )));
                        }
                        let key_count = found_keys.len();
                        if key_count == 1
                            && let Some(path) = found_keys.first()
                        {
                            var_keys.push((var, path));
                        } else if key_count == 0 {
                            return Err(VisitError::KeyError(KeyError::InnerKeyError(
                                InnerKeyError::InnerKeyNotFound,
                            )));
                        } else {
                            return Err(VisitError::KeyError(KeyError::TooManyKeys));
                        }
                    }
                    Fields::Unit => panic!("Schemas cannot have unit types"),
                }
            }
            Ok(Key::EnumInner {
                variant_fields: var_keys,
            })
        }
        Data::Union(_) => {
            panic!("Schemas can only be Structs or enums")
        }
    }
}

pub fn find_keys<'ast>(
    item: &'ast DeriveInput,
    schema_enum_gen: &SchemaEnumGenerator,
) -> Result<Key<'ast>, VisitError> {
    let registry = item
        .attrs
        .iter()
        .find(|att| att.path().is_ident("__netabase_registery"));
    match (find_outer_key_fn_path(&item), find_inner_key(&item.data)) {
        (Ok(_), Ok(_)) => Err(VisitError::KeyError(KeyError::TooManyKeys)),
        (Ok(out_key), Err(_)) => Ok(out_key),
        (Err(_), Ok(in_key)) => Ok(in_key),
        (Err(_), Err(_)) => {
            if let Some(att) = registry {
                Ok(Key::Registry(att))
            } else {
                Err(VisitError::KeyError(KeyError::KeyNotFound))
            }
        }
    }
}
