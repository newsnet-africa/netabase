use std::str::FromStr;

use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{Path, parse_quote};

use crate::SchemaModuleVisitor;

/// Generate the appropriate path for netabase traits - always use ::netabase_store:: for external usage
pub fn netabase_traits_path() -> Path {
    parse_quote! { ::netabase_store::traits }
}

/// Generate the appropriate path for netabase relational module - always use ::netabase_store:: for external usage
pub fn netabase_relational_path() -> Path {
    parse_quote! { ::netabase_store::relational }
}

/// Generate the appropriate path for netabase errors module - always use ::netabase_store:: for external usage
pub fn netabase_errors_path() -> Path {
    parse_quote! { ::netabase_store::errors }
}

/// Generate NetabaseModel trait path
pub fn netabase_model_trait_path() -> Path {
    let mut base = netabase_traits_path();
    base.segments.push(parse_quote! { NetabaseModel });
    base
}

/// Generate NetabaseModelKey trait path
pub fn netabase_model_key_trait_path() -> Path {
    let mut base = netabase_traits_path();
    base.segments.push(parse_quote! { NetabaseModelKey });
    base
}

/// Generate NetabaseSecondaryKeys trait path
pub fn netabase_secondary_keys_trait_path() -> Path {
    let mut base = netabase_traits_path();
    base.segments.push(parse_quote! { NetabaseSecondaryKeys });
    base
}

/// Generate NetabaseRelationalKeys trait path
pub fn netabase_relational_keys_trait_path() -> Path {
    let mut base = netabase_traits_path();
    base.segments.push(parse_quote! { NetabaseRelationalKeys });
    base
}

/// Generate RelationalLink type path
pub fn relational_link_type_path() -> Path {
    let mut base = netabase_relational_path();
    base.segments.push(parse_quote! { RelationalLink });
    base
}

/// Generate NetabaseError type path
pub fn netabase_error_type_path() -> Path {
    let mut base = netabase_errors_path();
    base.segments.push(parse_quote! { NetabaseError });
    base
}

// Removed unused traits NetabaseItemStruct and NetabaseItem

impl SchemaModuleVisitor {
    pub fn format_paths(&self) -> Vec<((TokenStream, TokenStream), (TokenStream, TokenStream))> {
        self.k_v_pairs
            .iter()
            .map(|(k, v)| {
                eprintln!(
                    "Formatting Paths: {:?}, {:?}",
                    k.to_token_stream().to_string(),
                    v.to_token_stream().to_string()
                );
                (
                    (
                        TokenStream::from_str(
                            &k.iter()
                                .map(|p| p.to_token_stream().to_string())
                                .collect::<Vec<String>>()
                                .join("::"),
                        )
                        .unwrap(),
                        k.last().to_token_stream(),
                    ),
                    (
                        TokenStream::from_str(
                            &v.iter()
                                .map(|p| p.to_token_stream().to_string())
                                .collect::<Vec<String>>()
                                .join("::"),
                        )
                        .unwrap(),
                        v.last().to_token_stream(),
                    ),
                )
            })
            .collect()
    }
}
