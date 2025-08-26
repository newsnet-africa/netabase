mod key_validator;
pub(super) mod validation_error;

use proc_macro2::Span;
use syn::{DeriveInput, Field, Ident, Path, Signature, Type, Variant, visit::Visit};

use crate::visitors::{
    key_validator::{find_inner_key, find_keys, find_outer_key_fn_path},
    validation_error::VisitError,
};

pub enum Key<'ast> {
    Outer {
        sig: Signature,
    },
    StructInner {
        field: &'ast Field,
        tuple_number: Option<usize>,
    },
    EnumInner {
        variant_fields: Vec<(&'ast Variant, &'ast Field)>,
    },
}
impl<'ast> Key<'ast> {
    pub fn ident(schema_ident: &Ident) -> Ident {
        let mut new_name = schema_ident.to_string();
        new_name.push_str("Key");
        Ident::new(&new_name, Span::call_site())
    }
}

#[derive(Default)]
enum SchemaValidatorType<'ast> {
    #[default]
    NotInitiated,
    Invalid,
    Struct {
        key: Key<'ast>,
        derive_input: &'ast DeriveInput,
    },
    Enum {
        key: Key<'ast>,
        derive_input: &'ast DeriveInput,
    },
}

#[derive(Default)]
pub struct SchemaValidator<'ast>(SchemaValidatorType<'ast>);

impl<'ast> SchemaValidator<'ast> {
    pub fn key(&self) -> Result<&Key<'ast>, VisitError> {
        match &self.0 {
            SchemaValidatorType::NotInitiated => Err(VisitError::InvalidSchemaType),
            SchemaValidatorType::Invalid => Err(VisitError::InvalidSchemaType),
            SchemaValidatorType::Struct {
                key,
                derive_input: _,
            } => Ok(&key),
            SchemaValidatorType::Enum {
                key,
                derive_input: _,
            } => Ok(&key),
        }
    }
    pub fn derive_input(&self) -> Result<&'ast DeriveInput, VisitError> {
        match &self.0 {
            SchemaValidatorType::NotInitiated => Err(VisitError::InvalidSchemaType),
            SchemaValidatorType::Invalid => Err(VisitError::InvalidSchemaType),
            SchemaValidatorType::Struct {
                key: _,
                derive_input,
            } => Ok(&derive_input),
            SchemaValidatorType::Enum {
                key: _,
                derive_input,
            } => Ok(&derive_input),
        }
    }

    pub fn ident(&self) -> Result<&'ast syn::Ident, VisitError> {
        match &self.0 {
            SchemaValidatorType::NotInitiated => Err(VisitError::InvalidSchemaType),
            SchemaValidatorType::Invalid => Err(VisitError::InvalidSchemaType),
            SchemaValidatorType::Struct { key, derive_input } => Ok(&derive_input.ident),
            SchemaValidatorType::Enum { key, derive_input } => Ok(&derive_input.ident),
        }
    }
}

impl<'ast> Visit<'ast> for SchemaValidator<'ast> {
    fn visit_derive_input(&mut self, i: &'ast DeriveInput) {
        let key = find_keys(&i).expect("Fix later");
        match &i.data {
            syn::Data::Struct(_data_struct) => {
                self.0 = SchemaValidatorType::Struct {
                    key,
                    derive_input: i,
                }
            }
            syn::Data::Enum(_data_enum) => {
                self.0 = SchemaValidatorType::Enum {
                    key,
                    derive_input: i,
                }
            }
            syn::Data::Union(_data_union) => panic!("Unions are invalid"),
        }
    }
}
