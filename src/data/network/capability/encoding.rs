//! Capability Encoding
//!
//! Provides encodings that omit sensitive information for transmission.
//! During PAI, the namespace/subspace/path context is already established,
//! so we can omit that data from capability transmissions.

use serde::{Deserialize, Serialize};

use crate::data::util::encryption::{NamespacePublicKey, NamespaceSignature};

use super::{
    area::{Area, PathConstraint, SubspaceConstraint, TimeRange},
    meadowcap::{
        AccessMode, CapabilityDelegation, CommunalCapability, McCapability, OwnedCapability,
        UserPublicKey, UserSignature,
    },
};

/// Encoded capability for transmission
///
/// Omits namespace_id, subspace_id, and path - these are inferred from PAI context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncodedCapability {
    /// Whether this is communal or owned
    pub is_communal: bool,
    /// Access mode
    pub access_mode: AccessMode,
    /// For communal: the user_key; for owned: user_key + initial_authorisation
    pub authority: EncodedAuthority,
    /// Encoded delegations (areas are relative)
    pub delegations: Vec<EncodedDelegation>,
}

/// Authority section of encoded capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EncodedAuthority {
    /// Communal: just the user key
    Communal { user_key: UserPublicKey },
    /// Owned: user key + namespace signature
    Owned {
        user_key: UserPublicKey,
        initial_authorisation: Vec<u8>,
    },
}

/// Encoded delegation (omits absolute area, uses relative encoding)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncodedDelegation {
    /// Relative area encoding (relative to context or previous delegation)
    pub relative_area: RelativeArea,
    /// User receiving the delegation
    pub user: UserPublicKey,
    /// Signature bytes
    pub signature: Vec<u8>,
}

/// Relative area encoding
///
/// Instead of encoding the full area, we encode only what differs from the context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelativeArea {
    /// If present, overrides the subspace from context
    pub subspace_override: Option<SubspaceConstraint>,
    /// Additional path components beyond context
    pub path_suffix: Vec<Vec<u8>>,
    /// Time range (always included as it can't be inferred)
    pub times: TimeRange,
}

impl RelativeArea {
    /// Encode an area relative to a context area
    pub fn encode(area: &Area, context: &Area) -> Self {
        // Subspace override only needed if different from context
        let subspace_override = if area.subspace != context.subspace {
            Some(area.subspace.clone())
        } else {
            None
        };

        // Path suffix: components beyond context path
        let path_suffix = if area.path.components.len() > context.path.components.len() {
            area.path.components[context.path.components.len()..].to_vec()
        } else {
            Vec::new()
        };

        Self {
            subspace_override,
            path_suffix,
            times: area.times.clone(),
        }
    }

    /// Decode an area given a context
    pub fn decode(&self, context: &Area) -> Area {
        let subspace = self
            .subspace_override
            .clone()
            .unwrap_or_else(|| context.subspace.clone());

        let mut path_components = context.path.components.clone();
        path_components.extend(self.path_suffix.clone());

        Area {
            subspace,
            path: PathConstraint::new(path_components),
            times: self.times.clone(),
        }
    }
}

impl EncodedCapability {
    /// Encode a McCapability for transmission
    ///
    /// The context_area represents what's already known from PAI.
    pub fn encode(cap: &McCapability, context_area: &Area) -> Self {
        match cap {
            McCapability::Communal(c) => Self::encode_communal(c, context_area),
            McCapability::Owned(o) => Self::encode_owned(o, context_area),
        }
    }

    fn encode_communal(cap: &CommunalCapability, _context_area: &Area) -> Self {
        let mut prev_area = Area::subspace(cap.user_key.clone());

        let delegations = cap
            .delegations
            .iter()
            .map(|d| {
                let relative = RelativeArea::encode(&d.area, &prev_area);
                prev_area = d.area.clone();
                EncodedDelegation {
                    relative_area: relative,
                    user: d.user.clone(),
                    signature: d.signature.as_bytes().to_vec(),
                }
            })
            .collect();

        Self {
            is_communal: true,
            access_mode: cap.access_mode,
            authority: EncodedAuthority::Communal {
                user_key: cap.user_key.clone(),
            },
            delegations,
        }
    }

    fn encode_owned(cap: &OwnedCapability, _context_area: &Area) -> Self {
        let mut prev_area = Area::full();

        let delegations = cap
            .delegations
            .iter()
            .map(|d| {
                let relative = RelativeArea::encode(&d.area, &prev_area);
                prev_area = d.area.clone();
                EncodedDelegation {
                    relative_area: relative,
                    user: d.user.clone(),
                    signature: d.signature.as_bytes().to_vec(),
                }
            })
            .collect();

        Self {
            is_communal: false,
            access_mode: cap.access_mode,
            authority: EncodedAuthority::Owned {
                user_key: cap.user_key.clone(),
                initial_authorisation: cap.initial_authorisation.as_bytes().to_vec(),
            },
            delegations,
        }
    }

    /// Decode a capability given the namespace context
    pub fn decode(self, namespace_key: NamespacePublicKey) -> McCapability {
        if self.is_communal {
            self.decode_communal(namespace_key)
        } else {
            self.decode_owned(namespace_key)
        }
    }

    fn decode_communal(self, namespace_key: NamespacePublicKey) -> McCapability {
        let user_key = match self.authority {
            EncodedAuthority::Communal { user_key } => user_key,
            _ => panic!("Expected communal authority"),
        };

        let mut prev_area = Area::subspace(user_key.clone());

        let delegations = self
            .delegations
            .into_iter()
            .map(|d| {
                let area = d.relative_area.decode(&prev_area);
                prev_area = area.clone();
                CapabilityDelegation {
                    area,
                    user: d.user,
                    signature: UserSignature::new(d.signature),
                }
            })
            .collect();

        McCapability::Communal(CommunalCapability {
            access_mode: self.access_mode,
            namespace_key,
            user_key,
            delegations,
        })
    }

    fn decode_owned(self, namespace_key: NamespacePublicKey) -> McCapability {
        let (user_key, initial_authorisation) = match self.authority {
            EncodedAuthority::Owned {
                user_key,
                initial_authorisation,
            } => (user_key, NamespaceSignature::new(initial_authorisation)),
            _ => panic!("Expected owned authority"),
        };

        let mut prev_area = Area::full();

        let delegations = self
            .delegations
            .into_iter()
            .map(|d| {
                let area = d.relative_area.decode(&prev_area);
                prev_area = area.clone();
                CapabilityDelegation {
                    area,
                    user: d.user,
                    signature: UserSignature::new(d.signature),
                }
            })
            .collect();

        McCapability::Owned(OwnedCapability {
            access_mode: self.access_mode,
            namespace_key,
            user_key,
            initial_authorisation,
            delegations,
        })
    }

    /// Get the receiver without full decoding
    pub fn receiver(&self) -> &UserPublicKey {
        self.delegations
            .last()
            .map(|d| &d.user)
            .unwrap_or_else(|| match &self.authority {
                EncodedAuthority::Communal { user_key } => user_key,
                EncodedAuthority::Owned { user_key, .. } => user_key,
            })
    }
}

/// Authorization token for writing entries (Meadowcap)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeadowcapAuthorisationToken {
    /// The capability proving write access
    pub capability: EncodedCapability,
    /// Signature over the entry by the capability receiver
    pub signature: Vec<u8>,
}

impl MeadowcapAuthorisationToken {
    pub fn new(capability: EncodedCapability, signature: Vec<u8>) -> Self {
        Self {
            capability,
            signature,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use libp2p::PeerId;

    #[test]
    fn test_relative_area_roundtrip() {
        let user = PeerId::random();
        let context = Area::subspace(user.clone());

        let area = Area::subspace(user)
            .with_path(PathConstraint::new(vec![b"a".to_vec(), b"b".to_vec()]))
            .with_times(TimeRange::new(Some(100), Some(200)));

        let relative = RelativeArea::encode(&area, &context);
        let decoded = relative.decode(&context);

        assert_eq!(area.path.components, decoded.path.components);
        assert_eq!(area.times, decoded.times);
    }

    #[test]
    fn test_encode_decode_communal() {
        let namespace_key = NamespacePublicKey::new([1u8; 32]);
        let user_key = PeerId::random();

        let cap = CommunalCapability::new_root(AccessMode::Read, namespace_key.clone(), user_key.clone());

        let mc_cap = McCapability::Communal(cap);
        let context = Area::full();

        let encoded = EncodedCapability::encode(&mc_cap, &context);
        let decoded = encoded.decode(namespace_key.clone());

        assert_eq!(decoded.access_mode(), AccessMode::Read);
        assert_eq!(decoded.receiver(), &user_key);
    }
}
