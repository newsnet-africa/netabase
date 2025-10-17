use thiserror::Error;

#[derive(Error, Debug)]
pub enum MacroError {
    #[error("There was an error parsing a derive Macro: {0:?}")]
    Derive(#[from] DeriveError),
}

#[derive(Error, Debug)]
pub enum DeriveError {
    #[error("Failed to parse Netabase Model")]
    Model(#[from] NetabaseModelDeriveError),
}

#[derive(Error, Debug)]
pub enum NetabaseModelDeriveError {
    #[error("Could not find a primary Key")]
    PrimaryKeyNotFound,
    #[error("Incorrect Model type")]
    IncorrectModelType,
    #[error("Failed to parse Link path")]
    LinkPath(#[from] LinkPathError),
    #[error("Macro Visitor Error")]
    MacroVisitorError,
}

#[derive(Error, Debug)]
pub enum LinkPathError {
    #[error("There was an eror parsing the link attribute")]
    Parse(proc_macro2::TokenStream),
    #[error("The link attribute was not a metalist")]
    IncorrectAttribute,
}
