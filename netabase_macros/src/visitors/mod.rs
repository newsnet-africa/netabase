mod key_validator;
pub mod schema_counter;
pub(super) mod validation_error;

use quote::ToTokens;
pub use schema_counter::SchemaCounterVisitor;

use proc_macro2::{Span, TokenStream};
use syn::{
    Attribute, DataEnum, DeriveInput, Field, Ident, ItemEnum, ItemImpl, Signature, Type, Variant,
    parse_quote, visit::Visit,
};

use crate::{
    SchemaEnumGenerator,
    generators::GenerationError,
    visitors::{key_validator::find_keys, validation_error::VisitError},
};

#[derive(Debug)]
pub enum Key<'ast> {
    Registry(&'ast Attribute),
    Outer {
        sig: Box<Signature>,
    },
    StructInner {
        field: &'ast Field,
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

#[derive(Default, Debug)]
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

#[derive(Default, Debug)]
pub struct SchemaValidator<'ast>(SchemaValidatorType<'ast>, pub SchemaEnumGenerator<'ast>);

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
            SchemaValidatorType::Struct {
                key: _,
                derive_input,
            } => Ok(&derive_input.ident),
            SchemaValidatorType::Enum {
                key: _,
                derive_input,
            } => Ok(&derive_input.ident),
        }
    }
}

impl<'ast> Visit<'ast> for SchemaValidator<'ast> {
    fn visit_derive_input(&mut self, i: &'ast DeriveInput) {
        let key = match find_keys(&i, &self.1) {
            Ok(key) => key,
            Err(_) => {
                self.0 = SchemaValidatorType::Invalid;
                return;
            }
        };

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
            syn::Data::Union(_data_union) => {
                self.0 = SchemaValidatorType::Invalid;
            }
        }
    }
}

#[derive(Default)]
pub struct RegistryVisitor<'ast> {
    data_enum: Option<&'ast DataEnum>,
    ident: Option<&'ast Ident>,
    variants: Vec<&'ast Variant>,
}

impl<'ast> Visit<'ast> for RegistryVisitor<'ast> {
    fn visit_derive_input(&mut self, i: &'ast syn::DeriveInput) {
        match &i.data {
            syn::Data::Enum(data_enum) => {
                self.variants = data_enum.variants.iter().collect();
                self.data_enum = Some(data_enum);
                self.ident = Some(&i.ident)
            }
            _ => panic!("Only enums can be registries"),
        }
    }
}

impl<'ast> RegistryVisitor<'ast> {
    pub fn generate_keys_registry(&self) -> Result<(ItemEnum, ItemImpl), GenerationError> {
        let new_name = {
            let mut old = self.ident.unwrap().to_string();
            old.push_str("Key");
            Ident::new(&old, proc_macro2::Span::call_site())
        };

        let new_variants = self
            .variants
            .clone()
            .into_iter()
            .map(|v| {
                // clone the variant and modify the last unnamed type's ident if present
                let mut v = (*v).clone();

                // If last field of an unnamed tuple variant exists and is a path type, append "Key" to its segment ident.
                if let syn::Fields::Unnamed(ref mut fields) = v.fields {
                    if let Some(last_field) = fields.unnamed.last_mut() {
                        if let syn::Type::Path(ref mut ty_path) = last_field.ty {
                            if let Some(last_seg) = ty_path.path.segments.last_mut() {
                                let mut ident = last_seg.ident.to_string();
                                ident.push_str("Key");
                                last_seg.ident = Ident::new(&ident, proc_macro2::Span::call_site());
                            }
                        }
                    }
                }

                // Rebuild the variant tokens correctly to avoid nested parentheses.
                let id = &v.ident;
                let tokens = match &v.fields {
                    syn::Fields::Unnamed(fields_unnamed) => {
                        // collect inner types and emit a single parenthesized tuple
                        let types: Vec<_> = fields_unnamed.unnamed.iter().map(|f| &f.ty).collect();
                        quote::quote! { #id ( #(#types),* ) }
                    }
                    syn::Fields::Named(fields_named) => {
                        let named = &fields_named.named;
                        quote::quote! { #id { #named } }
                    }
                    syn::Fields::Unit => quote::quote! { #id },
                };

                tokens
            })
            .collect::<Vec<TokenStream>>();

        Ok((
            parse_quote! {
                #[derive(Clone, Debug, ::macro_exports::__netabase_derive_more::From, ::macro_exports::__netabase_derive_more::TryInto)]
                #[::macro_exports::__netabase_enum_unwrapper::unique_try_froms()]
                pub enum #new_name {
                    #(#new_variants),*
                }
            },
            parse_quote! {
                impl netabase::netabase_trait::NetabaseRegistryKey for #new_name {

                }
            },
        ))
    }
}
