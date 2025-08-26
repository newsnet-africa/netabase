use std::{
    error::Error,
    fmt::{Debug, Display, Formatter, write},
};
#[derive(Debug)]
pub enum VisitError {
    KeyError(KeyError),
    ParseError(syn::Error),
    InvalidSchemaType,
}

#[derive(Debug)]
pub enum KeyError {
    TooManyKeys,
    KeyNotFound,
    InvalidSchema,
    InnerKeyError(InnerKeyError),
    OuterKeyError(OuterKeyError),
}

#[derive(Debug)]
pub enum OuterKeyError {
    OuterKeyNotFound,
    ReturnTypeNotFound,
    ArgumentReceiverNotFound,
}
#[derive(Debug)]
pub enum InnerKeyError {
    InnerKeyNotFound,
}

impl Display for VisitError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            VisitError::KeyError(key_count_error) => match key_count_error {
                KeyError::TooManyKeys => {
                    write!(f, "Too many keys")
                }
                KeyError::InnerKeyError(inner_key_error) => todo!(),
                KeyError::OuterKeyError(outer_key_error) => todo!(),
                KeyError::InvalidSchema => todo!(),
                KeyError::KeyNotFound => todo!(),
            },
            VisitError::InvalidSchemaType => {
                write!(
                    f,
                    " Invalid schema type. Only structs and enums are allowed"
                )
            }
            VisitError::ParseError(error) => todo!(),
        }
    }
}

impl Error for VisitError {}
