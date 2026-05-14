// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: AGPL-3.0-only

//! Principal types for identity and capability possession.
//!
//! A [`Principal`] wraps an `amari_enumerative::Namespace` — the geometric
//! representation of an identity's access space within the Grassmannian.
//! Capabilities are stored directly in the namespace (no dual storage),
//! and metadata (descriptions, kinds) lives in the controller's capability
//! registry.
//!
//! # Identity Model
//!
//! Identity is external. A [`PrincipalId`] is an opaque string — callers
//! map it to whatever identity system they use (JWT subject, OAuth client_id,
//! database user ID). Schubert does not authenticate; it authorizes.
//!
//! # Namespace
//!
//! Each principal occupies a position in the Grassmannian. When capabilities
//! are granted, amari [`Namespace::grant`] adds them to the namespace's
//! capability list. Access checks intersect the namespace position with
//! required Schubert classes to produce an [`AccessDecision`](crate::AccessDecision).

use amari_enumerative::{Namespace, NamespaceBuilder};
use std::fmt;

/// Unique identifier for a principal (user, service, token).
///
/// An opaque string wrapper. Identity is external — map this to your
/// authentication system's subject identifiers.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PrincipalId(pub String);

impl PrincipalId {
    /// Create a new principal ID.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
    /// Return the inner string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PrincipalId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for PrincipalId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}
impl From<String> for PrincipalId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

/// A principal — an identity that holds capabilities.
///
/// Wraps an `amari_enumerative::Namespace` which stores the Schubert
/// conditions defining this principal's geometric access space.
/// Capability metadata (labels, descriptions, kinds) is held by the
/// [`AccessController`](crate::AccessController) registry.
///
/// # Grant Ordering
///
/// Capabilities are stored in the namespace in grant order. The
/// namespace preserves insertion order for audit and stability analysis.
///
/// # Serialization
///
/// When the `serde` feature is enabled, the `namespace` field is skipped
/// during serialization. The namespace must be reconstructed from capability
/// grants after deserializing the full controller state.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Principal {
    /// Unique identifier for this principal.
    pub id: PrincipalId,
    /// The namespace representing this principal's access space.
    #[cfg_attr(feature = "serde", serde(skip))]
    pub namespace: Namespace,
    /// Capability IDs that have been granted (in grant order).
    ///
    /// Mirrors the namespace capability list for serialization.
    /// The namespace itself is skipped during serde.
    pub granted_capability_ids: Vec<String>,
    /// Unix timestamp of creation (milliseconds since epoch).
    pub created_at: u64,
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Principal {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        struct PrincipalData {
            id: PrincipalId,
            granted_capability_ids: Vec<String>,
            created_at: u64,
        }
        let data = PrincipalData::deserialize(deserializer)?;
        // Namespace is reconstructed externally via grants after deserialization.
        // Use a placeholder with the default Gr(2,4) — caller must rebuild via
        // AccessController::rebuild_principal_namespace().
        let namespace = NamespaceBuilder::new(data.id.as_str(), 2, 4)
            .build()
            .map_err(|e| serde::de::Error::custom(e.to_string()))?;
        Ok(Principal {
            id: data.id,
            namespace,
            granted_capability_ids: data.granted_capability_ids,
            created_at: data.created_at,
        })
    }
}

impl Principal {
    /// Create a new principal within a Grassmannian Gr(k,n).
    ///
    /// The principal starts with the identity position (empty partition —
    /// full access to the Grassmannian) and no granted capabilities.
    pub fn new(id: impl Into<PrincipalId>, k: usize, n: usize) -> crate::error::Result<Self> {
        let pid = id.into();
        let namespace = NamespaceBuilder::new(pid.as_str(), k, n)
            .build()
            .map_err(crate::error::SchubertError::Enumerative)?;
        Ok(Self {
            id: pid,
            namespace,
            granted_capability_ids: Vec::new(),
            created_at: now_millis(),
        })
    }

    /// Number of capabilities held by this principal.
    pub fn capability_count(&self) -> usize {
        self.namespace.capability_count()
    }

    /// Amari capability IDs in grant order.
    pub fn capability_ids(&self) -> Vec<amari_enumerative::CapabilityId> {
        self.namespace.capability_ids()
    }

    /// Check whether this principal holds a specific capability.
    ///
    /// Looks up the capability by string ID in the namespace's
    /// capability list. Returns `true` if found.
    pub fn holds(&self, id: &str) -> bool {
        let cid = amari_enumerative::CapabilityId::new(id);
        self.namespace.has_capability(&cid)
    }
}

/// Current time in milliseconds since Unix epoch.
///
/// Returns 0 on platforms without a system clock (e.g., `wasm32`,
/// or when the `std` feature is disabled).
pub(crate) fn now_millis() -> u64 {
    #[cfg(all(feature = "std", not(target_arch = "wasm32")))]
    {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0)
    }
    #[cfg(any(not(feature = "std"), target_arch = "wasm32"))]
    {
        0
    }
}
