//! Capability types for geometric access control.
//!
//! A [`Capability`] is a named Schubert condition — a geometric constraint
//! on the Grassmannian that reduces the space of valid configurations.
//! Each capability carries a partition defining a Schubert class σ_λ,
//! a semantic kind hint, and human-readable metadata.
//!
//! # Partition → Schubert Class
//!
//! | Partition | Schubert Class | Codimension | Typical Use |
//! |-----------|---------------|-------------|-------------|
//! | `[1]` | σ₁ | 1 | Read access |
//! | `[2]` | σ₂ | 2 | Write access |
//! | `[1,1]` | σ₁₁ | 2 | Read + audit |
//! | `[2,1]` | σ₂₁ | 3 | Manage |
//! | `[2,2]` | σ₂₂ | 4 | Admin (point class) |
//!
//! # Codimension Budget
//!
//! The total codimension of all granted capabilities must fit within the
//! Grassmannian dimension k(n−k). When it equals the dimension exactly,
//! access checks yield finite configuration counts. When it exceeds,
//! access is denied (overconstrained). When it falls short, access is
//! underconstrained (policy too permissive).

use crate::error::{Result, SchubertError};
use amari_enumerative::SchubertClass;
use std::fmt;

/// Unique identifier for a capability.
///
/// Lightweight string wrapper. Identity is determined by the ID alone —
/// two capabilities with the same ID are considered equal regardless
/// of partition, kind, or metadata.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CapabilityId(pub String);

impl CapabilityId {
    /// Create a new capability ID from a string.
    pub fn new(id: impl Into<String>) -> Self { Self(id.into()) }
    /// Return the inner string reference.
    pub fn as_str(&self) -> &str { &self.0 }
}

impl fmt::Display for CapabilityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{}", self.0) }
}

impl From<&str> for CapabilityId { fn from(s: &str) -> Self { Self(s.to_string()) } }
impl From<String> for CapabilityId { fn from(s: String) -> Self { Self(s) } }

/// Semantic classification of a capability.
///
/// Provides hints for humane policy design and reasoning. The kind
/// does not affect the mathematical computation — it's metadata for
/// policy authors and auditing tools.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CapabilityKind {
    /// Read-like capability. Typically codimension 1 (σ₁).
    ReadLike,
    /// Write-like capability. Typically codimension 2 (σ₂).
    WriteLike,
    /// Administrative capability. Typically the point class σ_{k×n} —
    /// a single specific configuration with maximum codimension.
    AdminLike,
    /// Custom capability with arbitrary partition semantics.
    Custom,
}

/// A named Schubert condition with human-readable metadata.
///
/// Capabilities are the atoms of the access control system. Each
/// represents a geometric constraint on the allowable configurations
/// of a principal within the Grassmannian.
///
/// # Example
///
/// ```
/// use schubert::{Capability, CapabilityKind};
///
/// let read = Capability::new(
///     "read:data", "Read data", vec![1], CapabilityKind::ReadLike,
/// );
/// assert_eq!(read.codimension(), 1);
/// ```
#[derive(Debug, Clone)]
pub struct Capability {
    /// Unique identifier for this capability.
    pub id: CapabilityId,
    /// Short human-readable label.
    pub label: String,
    /// Longer description of what this capability grants.
    pub description: String,
    /// The Schubert partition defining this condition.
    ///
    /// `vec![1]` = σ₁ (codimension 1), `vec![2,1]` = σ₂₁ (codimension 3).
    pub partition: Vec<usize>,
    /// Semantic kind for policy reasoning.
    pub kind: CapabilityKind,
}

impl Capability {
    /// Create a new capability with an empty description.
    pub fn new(
        id: impl Into<CapabilityId>,
        label: impl Into<String>,
        partition: Vec<usize>,
        kind: CapabilityKind,
    ) -> Self {
        Self { id: id.into(), label: label.into(), description: String::new(), partition, kind }
    }

    /// Create a new capability with a description.
    pub fn with_description(
        id: impl Into<CapabilityId>,
        label: impl Into<String>,
        description: impl Into<String>,
        partition: Vec<usize>,
        kind: CapabilityKind,
    ) -> Self {
        Self { id: id.into(), label: label.into(), description: description.into(), partition, kind }
    }

    /// The codimension of this capability — the sum of partition parts.
    ///
    /// This is the number of constraints imposed. Higher codimension
    /// means more restrictive. The point class σ_{k×n} has codimension
    /// equal to the Grassmannian dimension k(n−k).
    pub fn codimension(&self) -> usize {
        self.partition.iter().sum()
    }

    /// Convert this capability to an `amari_enumerative::SchubertClass`.
    ///
    /// Validates the partition against the given Grassmannian.
    pub fn to_schubert_class(&self, grassmannian: (usize, usize)) -> Result<SchubertClass> {
        SchubertClass::new(self.partition.clone(), grassmannian).map_err(|e| {
            SchubertError::InvalidPartition {
                partition: self.partition.clone(),
                k: grassmannian.0,
                n: grassmannian.1,
                reason: e.to_string(),
            }
        })
    }
}

impl PartialEq for Capability {
    fn eq(&self, other: &Self) -> bool { self.id == other.id }
}

impl Eq for Capability {}

impl std::hash::Hash for Capability {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) { self.id.hash(state); }
}
