use syn::{Ident, ItemStruct, PathSegment, Token, punctuated::Punctuated};

#[derive(Default)]
pub struct ModuleInfo<'a> {
    pub path: Punctuated<PathSegment, Token![::]>,
    pub models: Vec<&'a ItemStruct>,
    pub keys: Vec<Ident>,
}
