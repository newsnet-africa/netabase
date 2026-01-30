use netabase_store::prelude::NetabaseDefinition;

pub trait NetworkDefinition: NetabaseDefinition
where
    <Self as strum::IntoDiscriminant>::Discriminant: std::fmt::Debug,
    <Self as strum::IntoDiscriminant>::Discriminant: 'static,
{
    type DefinitionCapabilities;
}
