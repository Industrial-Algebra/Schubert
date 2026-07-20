// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

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
use amari_enumerative::{CapabilityId, Namespace, StabilityCondition, Wall, WallCrossingEngine};

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

    Ok(build_stability_report(
        principal_id,
        &principal.namespace,
        principal.capability_count(),
        walls,
        diagram,
    ))
}

/// Build a [`StabilityReport`] from a namespace and its computed walls/diagram.
///
/// Shared by [`analyze_stability`] (a principal in the controller) and
/// [`analyze_composed_stability`] (a synthesized composed namespace).
fn build_stability_report(
    principal_id: &PrincipalId,
    namespace: &Namespace,
    total_capabilities: usize,
    walls: Vec<Wall>,
    diagram: Vec<(f64, usize)>,
) -> StabilityReport {
    let mut seen = std::collections::HashSet::new();
    let phase_diagram = diagram
        .into_iter()
        .filter_map(|(trust, stable_count)| {
            if !seen.insert(stable_count) {
                return None;
            }
            let all_ids: Vec<String> = namespace
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

    StabilityReport {
        principal: principal_id.clone(),
        phase_diagram,
        walls,
        total_capabilities,
    }
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

/// Capabilities forming the operadic interface of a composition `A ∘_S B`.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct InterfaceCapabilities {
    /// Principal A's output capability (consumed by the composition).
    pub output: String,
    /// Principal B's input capability (consumed by the composition).
    pub input: String,
}

/// Wall-crossing stability analysis of an operadic composition `A ∘_S B`.
///
/// Computes the phase diagrams of the constituents (`P_A`, `P_B`) and of the
/// composed principal `C` (`P_C`), then tests whether `P_C` is the *additive*
/// combination of the constituents' retained-capability diagrams. This is the
/// empirical form of the Kontsevich–Soibelman-type question (Roadmap #17):
///
/// > Is the composed phase diagram `P_C` determined by `P_A` and `P_B`?
///
/// Where the relation holds (`is_additive`), composition is *predictable*.
/// Where it fails, the composed system exhibits **emergence**: stable-capability
/// counts not derivable from the constituents — the wall-crossing analogue of a
/// BPS bound state whose spectrum is not the union of its parts.
///
/// The composed principal `C` is synthesized from the *retained* capabilities
/// (each principal's non-interface capabilities); the access controller is not
/// mutated.
///
/// # Example
///
/// ```
/// use schubert::{AccessController, Capability, CapabilityKind, analyze_composed_stability};
///
/// let mut acl = AccessController::new(2, 4)?;
/// acl.register_capability(Capability::new("handoff", "Handoff", vec![1], CapabilityKind::ReadLike))?;
/// acl.register_capability(Capability::new("audit", "Audit", vec![2], CapabilityKind::WriteLike))?;
/// acl.register_capability(Capability::new("sign", "Sign", vec![1, 1], CapabilityKind::ReadLike))?;
///
/// let alice = acl.create_principal("alice")?; // outputs handoff, retains audit
/// acl.grant(&alice, "handoff")?;
/// acl.grant(&alice, "audit")?;
/// let bob = acl.create_principal("bob")?; // inputs handoff, retains sign
/// acl.grant(&bob, "handoff")?;
/// acl.grant(&bob, "sign")?;
///
/// let report = analyze_composed_stability(&acl, &alice, "handoff", &bob, "handoff")?;
/// // C retains audit + sign. is_additive holds iff P_C is determined by P_A and P_B.
/// assert_eq!(report.phase_diagram_composed.total_capabilities, 2);
/// assert_eq!(report.is_additive, report.non_additive_breakpoints.is_empty());
/// # Ok::<(), schubert::SchubertError>(())
/// ```
///
/// # Errors
///
/// Returns the same errors as [`compose`](crate::composition::compose) (principal
/// or capability lookup, composability). Returns [`SchubertError::Enumerative`]
/// if the composed namespace cannot be built.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ComposedStabilityReport {
    /// The left constituent (`A`).
    pub principal_a: PrincipalId,
    /// The right constituent (`B`).
    pub principal_b: PrincipalId,
    /// The consumed interface capabilities.
    pub interface: InterfaceCapabilities,
    /// Pushforward degree of the composition (from
    /// [`compose`](crate::composition::compose)).
    pub composition_multiplicity: u64,
    /// Phase diagram of `A` (all its capabilities).
    pub phase_diagram_a: StabilityReport,
    /// Phase diagram of `B` (all its capabilities).
    pub phase_diagram_b: StabilityReport,
    /// Phase diagram of the composed principal `C = A ∘_S B`.
    pub phase_diagram_composed: StabilityReport,
    /// `true` iff `P_C` equals the additive prediction from the constituents'
    /// retained capabilities (the KS-type relation holds; no emergence detected).
    pub is_additive: bool,
    /// Distinct trust levels at which `P_C` deviates from the additive
    /// prediction. Empty iff `is_additive`. These are the emergence signatures.
    pub non_additive_breakpoints: Vec<TrustLevel>,
}

/// Analyze the wall-crossing stability of an operadic composition.
///
/// See [`ComposedStabilityReport`] for the semantics of each field and the
/// Roadmap #17 question this instrument probes.
pub fn analyze_composed_stability(
    acl: &AccessController,
    principal_a: &PrincipalId,
    output_cap: &str,
    principal_b: &PrincipalId,
    input_cap: &str,
) -> Result<ComposedStabilityReport> {
    let composition =
        crate::composition::compose(acl, principal_a, output_cap, principal_b, input_cap)?;
    let report_a = analyze_stability(acl, principal_a)?;
    let report_b = analyze_stability(acl, principal_b)?;

    let a = acl
        .principal(principal_a)
        .ok_or_else(|| SchubertError::PrincipalNotFound(principal_a.to_string()))?;
    let b = acl
        .principal(principal_b)
        .ok_or_else(|| SchubertError::PrincipalNotFound(principal_b.to_string()))?;

    // C = (A minus its output capability) ∪ (B minus its input capability).
    let a_retained = retained_namespace(&a.namespace, output_cap);
    let b_retained = retained_namespace(&b.namespace, input_cap);
    let composed_ns = merge_namespaces(&a_retained, &b_retained)?;

    let grassmannian = acl.grassmannian();
    let engine = WallCrossingEngine::new(grassmannian);
    let walls_c = engine.compute_walls(&composed_ns);
    let diagram_c = engine.phase_diagram(&composed_ns);

    // KS-type additivity test: at each sampled trust level, does the composed
    // stable-count equal the sum of the retained constituents' stable-counts?
    let mut deviating: Vec<f64> = diagram_c
        .iter()
        .filter(|(trust, composed_count)| {
            let cond = StabilityCondition::standard(grassmannian, *trust);
            let predicted = cond.stable_count(&a_retained) + cond.stable_count(&b_retained);
            predicted != *composed_count
        })
        .map(|(trust, _)| *trust)
        .collect();
    deviating.sort_by(|x, y| x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal));
    deviating.dedup();
    let non_additive_breakpoints: Vec<TrustLevel> =
        deviating.into_iter().map(TrustLevel::new).collect();
    let is_additive = non_additive_breakpoints.is_empty();

    let composed_id = PrincipalId::new(format!("composed({} ∘ {})", principal_a.0, principal_b.0));
    let phase_diagram_composed = build_stability_report(
        &composed_id,
        &composed_ns,
        composed_ns.capabilities.len(),
        walls_c,
        diagram_c,
    );

    Ok(ComposedStabilityReport {
        principal_a: principal_a.clone(),
        principal_b: principal_b.clone(),
        interface: InterfaceCapabilities {
            output: output_cap.to_string(),
            input: input_cap.to_string(),
        },
        composition_multiplicity: composition.multiplicity,
        phase_diagram_a: report_a,
        phase_diagram_b: report_b,
        phase_diagram_composed,
        is_additive,
        non_additive_breakpoints,
    })
}

/// Clone a namespace with one capability removed (the retained view).
fn retained_namespace(namespace: &Namespace, remove_cap: &str) -> Namespace {
    let mut ns = namespace.clone();
    let cid = CapabilityId::new(remove_cap);
    let _ = ns.revoke(&cid);
    ns
}

/// Merge two namespaces: every capability of `additions` not already in `base`.
fn merge_namespaces(base: &Namespace, additions: &Namespace) -> Result<Namespace> {
    let mut merged = base.clone();
    for cap in &additions.capabilities {
        if !merged.has_capability(&cap.id) {
            merged.grant(cap.clone()).map_err(|e| {
                SchubertError::Enumerative(amari_enumerative::EnumerativeError::SchubertError(
                    e.to_string(),
                ))
            })?;
        }
    }
    Ok(merged)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AccessController, Capability, CapabilityKind};

    fn seeded_acl() -> AccessController {
        let mut acl = AccessController::new(2, 4).unwrap();
        acl.register_capability(Capability::new(
            "handoff",
            "Handoff",
            vec![1],
            CapabilityKind::ReadLike,
        ))
        .unwrap();
        acl.register_capability(Capability::new(
            "audit",
            "Audit",
            vec![2],
            CapabilityKind::WriteLike,
        ))
        .unwrap();
        acl.register_capability(Capability::new(
            "sign",
            "Sign",
            vec![1, 1],
            CapabilityKind::ReadLike,
        ))
        .unwrap();
        acl
    }

    #[test]
    fn composed_stability_reports_all_three_diagrams() {
        let mut acl = seeded_acl();
        let alice = acl.create_principal("alice").unwrap();
        let bob = acl.create_principal("bob").unwrap();
        acl.grant(&alice, "handoff").unwrap();
        acl.grant(&alice, "audit").unwrap();
        acl.grant(&bob, "handoff").unwrap();
        acl.grant(&bob, "sign").unwrap();

        let report = analyze_composed_stability(&acl, &alice, "handoff", &bob, "handoff").unwrap();

        assert_eq!(report.principal_a.0, "alice");
        assert_eq!(report.principal_b.0, "bob");
        assert_eq!(report.interface.output, "handoff");
        assert_eq!(report.interface.input, "handoff");
        // C retains audit + sign (the interface is consumed on both sides).
        assert_eq!(report.phase_diagram_composed.total_capabilities, 2);
        // multiplicity agrees with compose().
        let comp = crate::composition::compose(&acl, &alice, "handoff", &bob, "handoff").unwrap();
        assert_eq!(report.composition_multiplicity, comp.multiplicity);
        // constituent diagram agrees with analyze_stability.
        let sa = analyze_stability(&acl, &alice).unwrap();
        assert_eq!(
            report.phase_diagram_a.total_capabilities,
            sa.total_capabilities
        );
    }

    #[test]
    fn additivity_flag_is_self_consistent() {
        let mut acl = seeded_acl();
        let alice = acl.create_principal("alice").unwrap();
        let bob = acl.create_principal("bob").unwrap();
        acl.grant(&alice, "handoff").unwrap();
        acl.grant(&alice, "audit").unwrap();
        acl.grant(&bob, "handoff").unwrap();
        acl.grant(&bob, "sign").unwrap();

        let report = analyze_composed_stability(&acl, &alice, "handoff", &bob, "handoff").unwrap();
        // is_additive holds exactly when there are no emergence signatures.
        assert_eq!(
            report.is_additive,
            report.non_additive_breakpoints.is_empty()
        );
    }

    #[test]
    fn rejects_non_composable_pair() {
        let mut acl = seeded_acl();
        let alice = acl.create_principal("alice").unwrap();
        let bob = acl.create_principal("bob").unwrap();
        // alice does NOT hold the output capability -> compose() must fail.
        acl.grant(&alice, "audit").unwrap();
        acl.grant(&bob, "handoff").unwrap();

        let err = analyze_composed_stability(&acl, &alice, "handoff", &bob, "handoff");
        assert!(err.is_err());
    }
}
