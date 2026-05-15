// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: AGPL-3.0-only

//! Wall-crossing stability analysis.
//!
//! In geometric access control, capabilities have varying sensitivity to
//! trust degradation. A capability with high codimension relative to its
//! dimension becomes unstable at a lower trust threshold than one with low
//! codimension. The wall-crossing engine computes exactly where these
//! transitions occur.
//!
//! # Mathematical Background
//!
//! For each capability σ_λ, the stability condition is:
//!
//! ```text
//! phase = (1/π) · arctan(t · dim(σ_λ) / codim(σ_λ)) + 1/2
//! ```
//!
//! where t is the trust level. When phase drops below critical thresholds,
//! the capability transitions from stable to unstable. A **wall** is the
//! trust level where this transition occurs.
//!
//! The **phase diagram** is the piecewise-constant function mapping trust
//! levels to counts of stable capabilities. Breakpoints in the diagram
//! correspond to walls where individual capabilities cross stability
//! thresholds.
//!
//! # Practical Use
//!
//! Stability analysis answers: "If I only trust this principal to degree
//! t, which of their capabilities should I still honor?" Lower-codimension
//! capabilities (ReadLike) are more stable under trust degradation than
//! higher-codimension ones (AdminLike).

use crate::controller::AccessController;
use crate::error::{Result, SchubertError};
use crate::principal::PrincipalId;
use amari_enumerative::{StabilityCondition, Wall, WallCrossingEngine};

/// Trust level from 0.0 (no trust) to 1.0 (full trust).
///
/// At `FULL` (1.0), all capabilities are stable. As trust decreases,
/// higher-codimension capabilities become unstable first.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TrustLevel(pub f64);

impl TrustLevel {
    /// Full trust — all capabilities are stable.
    pub const FULL: Self = Self(1.0);
    /// No trust — no capabilities are stable.
    pub const NONE: Self = Self(0.0);

    /// Create a trust level, clamping to [0.0, 1.0].
    pub fn new(value: f64) -> Self {
        Self(value.clamp(0.0, 1.0))
    }
    /// Return the inner f64 value.
    pub fn value(&self) -> f64 {
        self.0
    }
}

impl Default for TrustLevel {
    fn default() -> Self {
        Self::FULL
    }
}

/// A breakpoint in the stability phase diagram.
///
/// At each breakpoint, the set of stable capabilities changes as
/// one or more capabilities cross their stability wall.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StabilityBreakpoint {
    /// The trust level at which this transition occurs.
    pub trust_level: TrustLevel,
    /// Capability IDs that remain stable above this threshold.
    pub stable_capabilities: Vec<String>,
    /// Capability IDs that become unstable below this threshold.
    pub unstable_capabilities: Vec<String>,
}

/// Complete stability analysis for a principal.
///
/// Contains the phase diagram (breakpoints where stability changes),
/// individual walls (per-capability transition points), and the total
/// number of capabilities analyzed.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StabilityReport {
    /// The principal analyzed.
    pub principal: PrincipalId,
    /// Breakpoints in descending trust order.
    pub phase_diagram: Vec<StabilityBreakpoint>,
    /// Individual walls detected for each capability.
    #[cfg_attr(feature = "serde", serde(skip, default))]
    pub walls: Vec<Wall>,
    /// Total capabilities held by the principal.
    pub total_capabilities: usize,
}

/// Analyze the stability of a principal's capabilities.
///
/// Computes the wall-crossing phase diagram: at what trust levels do
/// capabilities become unstable? Returns a [`StabilityReport`] with
/// the full diagram and individual walls.
///
/// # Example
///
/// ```
/// use schubert::{AccessController, Capability, CapabilityKind, analyze_stability, TrustLevel};
///
/// let mut acl = AccessController::new(2, 4)?;
/// acl.register_capability(Capability::new(
///     "read", "Read", vec![1], CapabilityKind::ReadLike,
/// ))?;
/// acl.register_capability(Capability::new(
///     "write", "Write", vec![2], CapabilityKind::WriteLike,
/// ))?;
///
/// let alice = acl.create_principal("alice")?;
/// acl.grant(&alice, "read")?;
/// acl.grant(&alice, "write")?;
///
/// let report = analyze_stability(&acl, &alice)?;
/// // report.phase_diagram — trust levels where stability changes
/// // report.walls — per-capability transition points
/// # Ok::<(), schubert::SchubertError>(())
/// ```
pub fn analyze_stability(
    acl: &AccessController,
    principal_id: &PrincipalId,
) -> Result<StabilityReport> {
    let principal = acl
        .principal(principal_id)
        .ok_or_else(|| SchubertError::PrincipalNotFound(principal_id.to_string()))?;

    let engine = WallCrossingEngine::new(acl.grassmannian());
    let walls = engine.compute_walls(&principal.namespace);
    let diagram = engine.phase_diagram(&principal.namespace);

    let mut seen = std::collections::HashSet::new();
    let breakpoints: Vec<StabilityBreakpoint> = diagram
        .into_iter()
        .filter_map(|(trust, stable_count)| {
            if !seen.insert(stable_count) {
                return None;
            }
            let all_ids: Vec<String> = principal
                .namespace
                .capabilities
                .iter()
                .map(|c| c.id.to_string())
                .collect();
            let stable = all_ids.iter().take(stable_count).cloned().collect();
            let unstable: Vec<String> = all_ids.iter().skip(stable_count).cloned().collect();
            Some(StabilityBreakpoint {
                trust_level: TrustLevel::new(trust),
                stable_capabilities: stable,
                unstable_capabilities: unstable,
            })
        })
        .collect();

    Ok(StabilityReport {
        principal: principal_id.clone(),
        phase_diagram: breakpoints,
        walls,
        total_capabilities: principal.capability_count(),
    })
}

/// Get the capabilities that remain stable at a given trust level.
///
/// Returns the capability IDs (as strings) that are still stable
/// when trust is degraded to the given level. Lower-codimension
/// capabilities survive longer.
///
/// # Example
///
/// ```
/// # use schubert::{AccessController, Capability, CapabilityKind, stable_capabilities_at, TrustLevel};
/// # let mut acl = AccessController::new(2, 4).unwrap();
/// # acl.register_capability(Capability::new("read", "", vec![1], CapabilityKind::ReadLike)).unwrap();
/// # let p = acl.create_principal("p").unwrap();
/// # acl.grant(&p, "read").unwrap();
/// let stable = stable_capabilities_at(&acl, &p, TrustLevel::new(0.5))?;
/// println!("Stable at 0.5 trust: {stable:?}");
/// # Ok::<(), schubert::SchubertError>(())
/// ```
pub fn stable_capabilities_at(
    acl: &AccessController,
    principal_id: &PrincipalId,
    trust: TrustLevel,
) -> Result<Vec<String>> {
    let principal = acl
        .principal(principal_id)
        .ok_or_else(|| SchubertError::PrincipalNotFound(principal_id.to_string()))?;

    let condition = StabilityCondition::standard(acl.grassmannian(), trust.value());
    let count = condition.stable_count(&principal.namespace);

    Ok(principal
        .namespace
        .capabilities
        .iter()
        .take(count)
        .map(|c| c.id.to_string())
        .collect())
}
