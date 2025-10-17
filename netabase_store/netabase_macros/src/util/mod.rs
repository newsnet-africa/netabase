use crate::errors::NetabaseModelDeriveError;
use syn::{
    Attribute, DeriveInput, Field, Fields, Ident, parse_quote,
};

pub mod visitor_utils;

pub fn append_ident(ident: &Ident, string: &str) -> Ident {
    let mut ident = ident.to_string();
    ident.push_str(string);
    let ident = Ident::new(&ident, proc_macro2::Span::call_site());
    parse_quote!(#ident)
}

pub fn extract_fields(input: &DeriveInput) -> &Fields {
    if let syn::Data::Struct(data_struct) = &input.data {
        &data_struct.fields
    } else {
        panic!(
            "Parse Error: {}",
            NetabaseModelDeriveError::IncorrectModelType
        );
    }
}

pub fn field_is_attribute<'a>(field: &'a Field, attribute: &'a str) -> Option<&'a Attribute> {
    field
        .attrs
        .iter()
        .find(|att| att.path().is_ident(attribute))
}
