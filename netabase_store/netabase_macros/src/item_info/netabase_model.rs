use syn::{
    Field, PathSegment, Token,
    punctuated::Punctuated,
};

pub struct ModelKeyInfo<'ast> {
    pub primary_keys: &'ast Field,
    pub secondary_keys: Vec<&'ast Field>,
}

pub struct ModelLinkInfo<'ast> {
    pub link_path: Punctuated<PathSegment, Token![::]>,
    pub link_field: &'ast Field,
}
