#![feature(impl_trait_in_assoc_type)]

pub mod network;
pub trait NetabaseRefCatalog<'a> {}
pub trait NetabaseCatalog {
    type RefCatelog<'a>: NetabaseRefCatalog<'a>;
}
