// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

use crate::capability::CapabilityId;
use amari_enumerative::IntersectionResult;
use std::fmt;

/// The result of an access control check.
///
/// Unlike traditional boolean allow/deny, this provides quantitative
/// information about the access decision.
///
/// # Variants
///
/// - `Granted` — access is allowed with a specific number of valid configurations
/// - `Impossible` — conditions are dimensionally compatible but geometrically
///   impossible (the σ₂·σ₁₁ = 0 case). This is distinct from `Denied`.
/// - `Denied` — too many conditions (codimension exceeds Grassmannian dimension)
/// - `Underconstrained` — too few conditions (access is a positive-dimensional variety)
///
/// # Example
///
/// ```
/// use schubert::AccessDecision;
///
/// let decision = AccessDecision::Granted {
///     configurations: 2,
///     path: schubert::ComputationPath::LittlewoodRichardson,
/// };
///
/// match decision {
///     AccessDecision::Granted { configurations, .. } => {
///         println!("Granted with {configurations} configurations");
///     }
///     AccessDecision::Impossible { conflicting } => {
///         println!("Impossible: {conflicting:?}");
///     }
///     AccessDecision::Denied => println!("Denied"),
///     AccessDecision::Underconstrained { dimension } => {
///         println!("Underconstrained (dim {dimension})");
///     }
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AccessDecision {
    /// Access granted with `configurations` valid configurations.
    Granted {
        /// Number of valid configurations satisfying all conditions.
        configurations: u64,
        /// Which computation path was used to compute this result.
        path: ComputationPath,
    },
    /// Conditions are dimensionally compatible but geometrically impossible.
    ///
    /// The sum of codimensions equals the Grassmannian dimension, but the
    /// Littlewood-Richardson coefficient is zero — no configuration can
    /// simultaneously satisfy all conditions.
    Impossible {
        /// Which capabilities created the impossibility.
        conflicting: Vec<CapabilityId>,
    },
    /// Access denied — too many conditions for the access space.
    ///
    /// The sum of codimensions exceeds the Grassmannian dimension.
    Denied,
    /// Access is underconstrained — the solution space has positive dimension.
    ///
    /// This indicates the policy is too permissive and should be tightened.
    Underconstrained {
        /// Dimension of the solution variety.
        dimension: usize,
    },
}

/// Which computation path was used to evaluate an access decision.
///
/// Re-exported from `amari_enumerative` for convenience, but typed as our
/// own enum so we don't leak library internals into the public API.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ComputationPath {
    /// Littlewood-Richardson rule (exact, classical).
    LittlewoodRichardson,
    /// Equivariant localization (Atiyah-Bott fixed-point formula).
    Localization,
    /// Tropical intersection theory.
    Tropical,
    /// Matroid-based independence check (polynomial time shortcut).
    Matroid,
}

impl fmt::Display for ComputationPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LittlewoodRichardson => write!(f, "Littlewood-Richardson"),
            Self::Localization => write!(f, "equivariant localization"),
            Self::Tropical => write!(f, "tropical intersection"),
            Self::Matroid => write!(f, "matroid independence"),
        }
    }
}

impl From<amari_enumerative::IntersectionResult> for AccessDecision {
    fn from(result: IntersectionResult) -> Self {
        match result {
            IntersectionResult::Finite(0) => Self::Impossible {
                conflicting: Vec::new(),
            },
            IntersectionResult::Finite(n) => Self::Granted {
                configurations: n,
                path: ComputationPath::LittlewoodRichardson,
            },
            IntersectionResult::PositiveDimensional { dimension, .. } => {
                Self::Underconstrained { dimension }
            }
            IntersectionResult::Empty => Self::Denied,
        }
    }
}

/// Optional context for access control decisions.
///
/// Extends the capability-based check with environmental factors:
/// - **resource**: Scope the check to a specific resource (e.g., "project:123")
/// - **time**: Current Unix timestamp in ms. When set, trust levels degrade
///   based on time deltas.
/// - **metadata**: Arbitrary key-value pairs for audit and custom logic.
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AccessContext {
    /// Specific resource this check is scoped to.
    pub resource: Option<String>,
    /// Current Unix timestamp in milliseconds.
    pub time: Option<u64>,
    /// Arbitrary context metadata for audit and custom logic.
    #[cfg_attr(feature = "serde", serde(default))]
    pub metadata: std::collections::HashMap<String, String>,
}

impl AccessContext {
    /// Create an empty context.
    pub fn empty() -> Self {
        Self::default()
    }

    /// Create a context with just a resource.
    pub fn for_resource(resource: impl Into<String>) -> Self {
        Self {
            resource: Some(resource.into()),
            ..Default::default()
        }
    }

    /// Create a context with just a timestamp.
    pub fn at_time(timestamp_ms: u64) -> Self {
        Self {
            time: Some(timestamp_ms),
            ..Default::default()
        }
    }
}
