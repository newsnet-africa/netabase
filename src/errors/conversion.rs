use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConversionError {
    #[error("There was an error with the macro conversion.")]
    MacroConversion,
    #[error("There was an error with the trait conversion.")]
    TraitConversion,
}
