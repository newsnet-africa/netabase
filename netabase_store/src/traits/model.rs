/// Trait for user-defined models that can be stored in the database.
///
/// This trait is automatically derived via the `#[derive(NetabaseModel)]` macro.
/// Models must have:
/// - A primary key field marked with `#[primary_key]`
/// - Optional secondary key fields marked with `#[secondary_key]`
pub trait NetabaseModelTrait: bincode::Encode + Sized + Clone + Send + Sync + 'static {
    /// The primary key type for this model
    type PrimaryKey: NetabaseModelTraitKey;

    /// The secondary keys enum for this model
    type SecondaryKeys: NetabaseModelTraitKey;

    /// The keys enum that wraps both primary and secondary keys
    type Keys: NetabaseModelTraitKey;

    /// Extract the primary key from the model instance
    fn primary_key(&self) -> Self::PrimaryKey;

    /// Extract all secondary keys from the model instance
    fn secondary_keys(&self) -> Vec<Self::SecondaryKeys>;

    /// Get the discriminant name for this model (used for tree names)
    fn discriminant_name() -> &'static str;
}

/// Marker trait for key types (both primary and secondary).
///
/// This trait is automatically implemented by the macro-generated key types.
pub trait NetabaseModelTraitKey:
    bincode::Encode + std::fmt::Debug + Clone + Send + Sync + 'static
{
}
