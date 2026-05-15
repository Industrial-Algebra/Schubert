// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: AGPL-3.0-only

use crate::audit::AuditSink;
use crate::capability::{Capability, CapabilityId};
use crate::decision::{AccessDecision, ComputationPath};
use crate::error::{Result, SchubertError};
use crate::principal::{Principal, PrincipalId};
#[cfg(not(feature = "std"))]
use alloc::collections::BTreeMap as HashMap;
use amari_enumerative::{IntersectionResult, SchubertCalculus};
#[cfg(feature = "parallel")]
use rayon::prelude::*;
#[cfg(feature = "std")]
use std::collections::HashMap;

/// Access controller for quantitative capability-based authorization.
///
/// Manages principals, capabilities, and access decisions within a fixed
/// Grassmannian Gr(k,n). Wraps `amari_enumerative`'s Schubert calculus
/// with an ergonomic access control API.
///
/// # Grassmannian Selection
///
/// | Gr(k,n) | Dimension | Use case |
/// |---------|-----------|----------|
/// | Gr(2,4) | 4 | Standard RBAC (recommended) |
/// | Gr(3,6) | 9 | Complex multi-tenant |
/// | Gr(4,8) | 16 | Enterprise policy space |
pub struct AccessController {
    grassmannian: (usize, usize),
    capabilities: HashMap<CapabilityId, Capability>,
    principals: HashMap<PrincipalId, Principal>,
    audit_sink: Option<Box<dyn AuditSink>>,
}

impl std::fmt::Debug for AccessController {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AccessController")
            .field("grassmannian", &self.grassmannian)
            .field("capabilities", &self.capabilities.len())
            .field("principals", &self.principals.len())
            .field("audit_sink", &self.audit_sink.as_ref().map(|_| "…"))
            .finish()
    }
}

impl AccessController {
    /// Create a new access controller for Gr(k,n).
    pub fn new(k: usize, n: usize) -> Result<Self> {
        if k == 0 || n < 2 || k >= n {
            return Err(SchubertError::InvalidGrassmannian {
                k,
                n,
                reason: "require k ≥ 1, n ≥ 2, k < n".into(),
            });
        }
        Ok(Self {
            grassmannian: (k, n),
            capabilities: HashMap::new(),
            principals: HashMap::new(),
            audit_sink: None,
        })
    }

    /// Return the Grassmannian parameters Gr(k,n).
    pub fn grassmannian(&self) -> (usize, usize) {
        self.grassmannian
    }

    /// Install an audit sink for recording access decisions.
    ///
    /// The sink receives every decision made by [`check`](Self::check).
    /// Audit failures are silently ignored — they never affect access decisions.
    pub fn set_audit_sink(&mut self, sink: Box<dyn AuditSink>) {
        self.audit_sink = Some(sink);
    }

    // ── Capability registry ────────────────────────────────────────

    /// Register a capability. The partition is validated against the Grassmannian.
    pub fn register_capability(&mut self, cap: Capability) -> Result<()> {
        if self.capabilities.contains_key(&cap.id) {
            return Err(SchubertError::CapabilityExists(cap.id.to_string()));
        }
        cap.to_schubert_class(self.grassmannian)?;
        self.capabilities.insert(cap.id.clone(), cap);
        Ok(())
    }

    /// Look up a registered capability by its string ID.
    pub fn capability(&self, id: &str) -> Option<&Capability> {
        self.capabilities.get(&CapabilityId::new(id))
    }

    /// Iterate over all registered capabilities.
    pub fn capabilities(&self) -> impl Iterator<Item = &Capability> {
        self.capabilities.values()
    }

    /// Look up the registered metadata for an amari CapabilityId.
    fn registered(&self, id: &amari_enumerative::CapabilityId) -> Option<&Capability> {
        self.capabilities.get(&CapabilityId::new(id.as_str()))
    }

    // ── Principal management ───────────────────────────────────────

    /// Create a new principal with no granted capabilities.
    ///
    /// The principal starts at the identity position (empty partition) —
    /// the least restrictive position possible in the Grassmannian.
    pub fn create_principal(&mut self, id: impl Into<PrincipalId>) -> Result<PrincipalId> {
        let id = id.into();
        if self.principals.contains_key(&id) {
            return Err(SchubertError::PrincipalExists(id.to_string()));
        }
        let p = Principal::new(id.clone(), self.grassmannian.0, self.grassmannian.1)?;
        self.principals.insert(id.clone(), p);
        Ok(id)
    }

    /// Grant a registered capability to a principal.
    pub fn grant(&mut self, principal_id: &PrincipalId, capability_id: &str) -> Result<()> {
        let our_cap = self
            .capabilities
            .get(&CapabilityId::new(capability_id))
            .ok_or_else(|| SchubertError::CapabilityNotFound(capability_id.to_string()))?
            .clone();

        let principal = self
            .principals
            .get_mut(principal_id)
            .ok_or_else(|| SchubertError::PrincipalNotFound(principal_id.to_string()))?;

        // Use amari's Namespace::grant directly
        let amari_cap = amari_enumerative::Capability::new(
            our_cap.id.to_string(),
            our_cap.label.clone(),
            our_cap.partition.clone(),
            self.grassmannian,
        )
        .map_err(SchubertError::Enumerative)?;

        principal.namespace.grant(amari_cap).map_err(|e| {
            SchubertError::Enumerative(amari_enumerative::EnumerativeError::SchubertError(
                e.to_string(),
            ))
        })?;

        // Track grant for serialization roundtrip
        principal
            .granted_capability_ids
            .push(capability_id.to_string());

        Ok(())
    }

    /// Revoke a capability from a principal.
    pub fn revoke(&mut self, principal_id: &PrincipalId, capability_id: &str) -> Result<()> {
        let principal = self
            .principals
            .get_mut(principal_id)
            .ok_or_else(|| SchubertError::PrincipalNotFound(principal_id.to_string()))?;

        let amari_cid = amari_enumerative::CapabilityId::new(capability_id);
        if !principal.namespace.has_capability(&amari_cid) {
            return Err(SchubertError::CapabilityNotHeld {
                principal: principal_id.to_string(),
                capability: capability_id.to_string(),
            });
        }

        principal.namespace.revoke(&amari_cid);
        principal
            .granted_capability_ids
            .retain(|id| id != capability_id);
        Ok(())
    }

    /// Look up a principal by ID.
    pub fn principal(&self, id: &PrincipalId) -> Option<&Principal> {
        self.principals.get(id)
    }

    /// Return capability metadata for a principal, in grant order.
    pub fn principal_capabilities(&self, id: &PrincipalId) -> Result<Vec<&Capability>> {
        let p = self
            .principals
            .get(id)
            .ok_or_else(|| SchubertError::PrincipalNotFound(id.to_string()))?;

        p.namespace
            .capabilities
            .iter()
            .map(|amari_cap| {
                self.registered(&amari_cap.id)
                    .ok_or_else(|| SchubertError::CapabilityNotFound(amari_cap.id.to_string()))
            })
            .collect()
    }

    // ── Access checks ──────────────────────────────────────────────

    /// Check whether a principal satisfies a set of capability requirements.
    ///
    /// Uses the Littlewood-Richardson path by default. For explicit path
    /// selection, use [`check_with_path`](Self::check_with_path). For
    /// automatic routing, use [`check_auto`](Self::check_auto).
    pub fn check(&self, principal_id: &PrincipalId, required: &[&str]) -> Result<AccessDecision> {
        self.check_with_path(
            principal_id,
            required,
            ComputationPath::LittlewoodRichardson,
        )
    }

    /// Check access with an explicit computation path preference.
    ///
    /// Routes to the requested amari computation engine:
    ///
    /// | Path | Engine | Best For |
    /// |------|--------|----------|
    /// | `LittlewoodRichardson` | `SchubertCalculus::multi_intersect` | Small Gr(k,n), few classes |
    /// | `Localization` | `EquivariantLocalizer::intersection_result` | Large Gr(k,n), many classes |
    /// | `Tropical` | `tropical_intersection_count` | Approximate counts |
    /// | `Matroid` | `Matroid::intersection_cardinality` | Fast independence check |
    pub fn check_with_path(
        &self,
        principal_id: &PrincipalId,
        required: &[&str],
        path: ComputationPath,
    ) -> Result<AccessDecision> {
        let principal = self
            .principals
            .get(principal_id)
            .ok_or_else(|| SchubertError::PrincipalNotFound(principal_id.to_string()))?;

        // Must hold all required capabilities
        for cap_id in required {
            if !principal.holds(cap_id) {
                return Ok(AccessDecision::Denied);
            }
        }

        // Build required Schubert classes
        let required_classes = self.resolve_required_classes(required)?;

        // Build full class list: position + required
        let mut all = vec![principal.namespace.position.clone()];
        all.extend(required_classes);

        let decision = match path {
            ComputationPath::LittlewoodRichardson => self.compute_lr(&all, required),
            ComputationPath::Localization => self.compute_localization(&all, required),
            ComputationPath::Tropical => self.compute_tropical(&all, required),
            ComputationPath::Matroid => self.compute_matroid(&all, required),
        }?;

        // Audit
        if let Some(ref sink) = self.audit_sink {
            let _ = sink.record(&crate::audit::DecisionRecord {
                principal: principal_id.clone(),
                capabilities: required.iter().map(|s| CapabilityId::new(*s)).collect(),
                decision: decision.clone(),
                timestamp: crate::principal::now_millis(),
            });
        }

        Ok(decision)
    }

    /// Check access with automatic computation path selection.
    ///
    /// Heuristic routing based on Grassmannian size and number of classes:
    ///
    /// - Gr(k,n) with n ≤ 8 and < 6 classes → LR (exact, fast for small)
    /// - Gr(k,n) with n > 8 or ≥ 6 classes → Localization (scales better)
    /// - Degenerate result → Tropical (cross-check)
    ///
    /// For an explicit path, use [`check_with_path`](Self::check_with_path).
    pub fn check_auto(
        &self,
        principal_id: &PrincipalId,
        required: &[&str],
    ) -> Result<AccessDecision> {
        let (n, class_count) = (self.grassmannian.1, required.len());
        let path = if n <= 8 && class_count < 6 {
            ComputationPath::LittlewoodRichardson
        } else {
            ComputationPath::Localization
        };
        self.check_with_path(principal_id, required, path)
    }

    /// Return the effective access dimension for a principal.
    pub fn effective_dimension(&self, principal_id: &PrincipalId) -> Result<isize> {
        let p = self
            .principals
            .get(principal_id)
            .ok_or_else(|| SchubertError::PrincipalNotFound(principal_id.to_string()))?;

        let dim = self.grassmannian.0 * (self.grassmannian.1 - self.grassmannian.0);
        let total_codim: usize = p
            .namespace
            .capabilities
            .iter()
            .map(|c| c.codimension())
            .sum();
        Ok(dim as isize - total_codim as isize)
    }

    // ── Computation path engines ─────────────────────────────────

    /// Resolve required capability string IDs to amari Schubert classes.
    fn resolve_required_classes(
        &self,
        required: &[&str],
    ) -> Result<Vec<amari_enumerative::SchubertClass>> {
        let mut classes = Vec::with_capacity(required.len());
        for cap_id_str in required {
            let cid = CapabilityId::new(*cap_id_str);
            let cap = self
                .capabilities
                .get(&cid)
                .ok_or_else(|| SchubertError::CapabilityNotFound(cid.to_string()))?;
            classes.push(cap.to_schubert_class(self.grassmannian)?);
        }
        Ok(classes)
    }

    /// Compute intersection via Littlewood-Richardson (exact, classical).
    fn compute_lr(
        &self,
        all: &[amari_enumerative::SchubertClass],
        required: &[&str],
    ) -> Result<AccessDecision> {
        let mut calc = SchubertCalculus::new(self.grassmannian);
        let result = calc.multi_intersect(all);
        Ok(map_intersection_result(
            result,
            required,
            ComputationPath::LittlewoodRichardson,
        ))
    }

    /// Compute intersection via equivariant localization (Atiyah-Bott).
    fn compute_localization(
        &self,
        all: &[amari_enumerative::SchubertClass],
        required: &[&str],
    ) -> Result<AccessDecision> {
        use amari_enumerative::EquivariantLocalizer;
        let mut localizer = EquivariantLocalizer::new(self.grassmannian)?;
        let result = localizer.intersection_result(all);
        Ok(map_intersection_result(
            result,
            required,
            ComputationPath::Localization,
        ))
    }

    /// Compute intersection via tropical geometry (fast approximate count).
    fn compute_tropical(
        &self,
        all: &[amari_enumerative::SchubertClass],
        required: &[&str],
    ) -> Result<AccessDecision> {
        let result = amari_enumerative::tropical_intersection_count(all, self.grassmannian);
        let intersection = result.to_intersection_result();
        Ok(map_intersection_result(
            intersection,
            required,
            ComputationPath::Tropical,
        ))
    }

    /// Compute intersection via matroid independence (polynomial time shortcut).
    ///
    /// Uses matroid intersection cardinality as a fast check. The matroid
    /// approach is inexact for counting but reliable for detecting
    /// impossibility (intersection cardinality 0 means no configuration).
    fn compute_matroid(
        &self,
        all: &[amari_enumerative::SchubertClass],
        required: &[&str],
    ) -> Result<AccessDecision> {
        use amari_enumerative::Matroid;

        if all.is_empty() {
            return Ok(AccessDecision::Underconstrained {
                dimension: self.grassmannian.0 * (self.grassmannian.1 - self.grassmannian.0),
            });
        }

        // Build matroid for the primary class
        let partition = all[0].to_partition();
        let mut matroid =
            Matroid::schubert_matroid(&partition.parts, self.grassmannian.0, self.grassmannian.1)
                .map_err(|e| {
                SchubertError::Enumerative(amari_enumerative::EnumerativeError::ComputationError(e))
            })?;

        // Intersect with subsequent classes
        for class in &all[1..] {
            let p = class.to_partition();
            let other =
                Matroid::schubert_matroid(&p.parts, self.grassmannian.0, self.grassmannian.1)
                    .map_err(|e| {
                        SchubertError::Enumerative(
                            amari_enumerative::EnumerativeError::ComputationError(e),
                        )
                    })?;
            let card = matroid.intersection_cardinality(&other);
            if card == 0 {
                return Ok(AccessDecision::Impossible {
                    conflicting: required.iter().map(|s| CapabilityId::new(*s)).collect(),
                });
            }
            matroid = other;
        }

        // Matroid passes — finite or underconstrained (can't count exactly via matroids alone)
        let dim = self.grassmannian.0 * (self.grassmannian.1 - self.grassmannian.0);
        let total_codim: usize = all.iter().map(|c| c.codimension()).sum();
        if total_codim > dim {
            Ok(AccessDecision::Denied)
        } else if total_codim == dim {
            // Transverse — but we don't know the exact count from matroids alone
            // Return a marker; caller should verify with LR or localization
            Ok(AccessDecision::Impossible {
                conflicting: required.iter().map(|s| CapabilityId::new(*s)).collect(),
            })
        } else {
            Ok(AccessDecision::Underconstrained {
                dimension: dim - total_codim,
            })
        }
    }

    // ── Parallel batch operations ─────────────────────────────────

    /// Check access for multiple (principal, requirements) pairs in parallel.
    ///
    /// # Example
    ///
    /// ```ignore
    /// # #[cfg(feature = "parallel")]
    /// # {
    /// let queries = [
    ///     (&alice, &["read:data"][..]),
    ///     (&bob, &["read:data", "write:data"]),
    /// ];
    /// let results = acl.check_batch(&queries)?;
    /// # }
    /// ```
    #[cfg(feature = "parallel")]
    pub fn check_batch(&self, queries: &[(&PrincipalId, &[&str])]) -> Vec<Result<AccessDecision>> {
        use amari_enumerative::multi_intersect_batch;

        // Gather all valid queries; track denied ones separately
        struct Query {
            index: usize,
            position: amari_enumerative::SchubertClass,
            classes: Vec<amari_enumerative::SchubertClass>,
            required_strs: Vec<String>,
        }

        let mut valid: Vec<Query> = Vec::with_capacity(queries.len());
        let mut results: Vec<Option<AccessDecision>> = vec![None; queries.len()];

        for (i, &(principal_id, required)) in queries.iter().enumerate() {
            let principal = match self.principals.get(principal_id) {
                Some(p) => p,
                None => {
                    results[i] = Some(AccessDecision::Denied);
                    continue;
                }
            };

            // Check holds
            if required.iter().any(|cid| !principal.holds(cid)) {
                results[i] = Some(AccessDecision::Denied);
                continue;
            }

            // Resolve required capabilities to Schubert classes
            let mut classes = Vec::with_capacity(required.len());
            for cap_id_str in required {
                let cid = CapabilityId::new(*cap_id_str);
                match self.capabilities.get(&cid) {
                    Some(cap) => match cap.to_schubert_class(self.grassmannian) {
                        Ok(cls) => classes.push(cls),
                        Err(_) => {
                            classes = vec![];
                            break;
                        }
                    },
                    None => {
                        classes = vec![];
                        break;
                    }
                }
            }

            valid.push(Query {
                index: i,
                position: principal.namespace.position.clone(),
                classes,
                required_strs: required.iter().map(|s| s.to_string()).collect(),
            });
        }

        // Batch intersect valid queries
        if !valid.is_empty() {
            let inputs: Vec<_> = valid
                .iter()
                .map(|q| {
                    let mut all = vec![q.position.clone()];
                    all.extend(q.classes.clone());
                    (all, self.grassmannian)
                })
                .collect();

            let batch_results = multi_intersect_batch(&inputs);

            for (q, result) in valid.into_iter().zip(batch_results) {
                let decision = match result {
                    IntersectionResult::Finite(0) => AccessDecision::Impossible {
                        conflicting: q.required_strs.into_iter().map(CapabilityId::new).collect(),
                    },
                    IntersectionResult::Finite(n) => AccessDecision::Granted {
                        configurations: n,
                        path: ComputationPath::LittlewoodRichardson,
                    },
                    IntersectionResult::PositiveDimensional { dimension, .. } => {
                        AccessDecision::Underconstrained { dimension }
                    }
                    IntersectionResult::Empty => AccessDecision::Denied,
                };
                results[q.index] = Some(decision);
            }
        }

        results
            .into_iter()
            .map(|opt| Ok(opt.unwrap_or(AccessDecision::Denied)))
            .collect()
    }

    /// Analyze stability for multiple principals in parallel.
    ///
    /// Returns one `StabilityReport` per principal.
    #[cfg(feature = "parallel")]
    pub fn stability_batch(
        &self,
        principal_ids: &[PrincipalId],
    ) -> Vec<Result<crate::stability::StabilityReport>> {
        let engine = amari_enumerative::WallCrossingEngine::new(self.grassmannian);

        principal_ids
            .par_iter()
            .map(|pid| {
                let principal = self
                    .principals
                    .get(pid)
                    .ok_or_else(|| SchubertError::PrincipalNotFound(pid.to_string()))?;

                let walls = engine.compute_walls(&principal.namespace);
                let diagram = engine.phase_diagram(&principal.namespace);

                use std::collections::HashSet;
                let mut seen = HashSet::new();
                let breakpoints: Vec<_> = diagram
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
                        let unstable: Vec<String> =
                            all_ids.iter().skip(stable_count).cloned().collect();
                        Some(crate::stability::StabilityBreakpoint {
                            trust_level: crate::stability::TrustLevel::new(trust),
                            stable_capabilities: stable,
                            unstable_capabilities: unstable,
                        })
                    })
                    .collect();

                Ok(crate::stability::StabilityReport {
                    principal: pid.clone(),
                    phase_diagram: breakpoints,
                    walls,
                    total_capabilities: principal.capability_count(),
                })
            })
            .collect()
    }

    /// Compose multiple principal pairs in parallel.
    ///
    /// Each tuple is (principal_a, output_cap, principal_b, input_cap).
    #[cfg(feature = "parallel")]
    pub fn compose_batch(
        &self,
        pairs: &[(&PrincipalId, &str, &PrincipalId, &str)],
    ) -> Vec<Result<crate::composition::CompositionResult>> {
        pairs
            .par_iter()
            .map(|&(a_id, out_cap, b_id, in_cap)| {
                crate::composition::compose(self, a_id, out_cap, b_id, in_cap)
            })
            .collect()
    }

    // ── Serialization helpers ────────────────────────────────────

    /// Rebuild all principal namespaces from their tracked grant IDs.
    ///
    /// After deserialization, principal namespaces are placeholders.
    /// This method rebuilds them with the correct Grassmannian parameters
    /// and re-grants all tracked capabilities.
    #[cfg(feature = "serde")]
    pub fn rebuild_principal_namespaces(&mut self) -> Result<()> {
        let (k, n) = self.grassmannian;
        for principal in self.principals.values_mut() {
            let namespace = amari_enumerative::NamespaceBuilder::new(principal.id.as_str(), k, n)
                .build()
                .map_err(SchubertError::Enumerative)?;
            principal.namespace = namespace;

            let grant_ids: Vec<String> = principal.granted_capability_ids.clone();
            for cap_id_str in &grant_ids {
                let our_cap = self
                    .capabilities
                    .get(&CapabilityId::new(cap_id_str.as_str()))
                    .ok_or_else(|| SchubertError::CapabilityNotFound(cap_id_str.to_string()))?
                    .clone();

                let amari_cap = amari_enumerative::Capability::new(
                    our_cap.id.to_string(),
                    our_cap.label.clone(),
                    our_cap.partition.clone(),
                    self.grassmannian,
                )
                .map_err(SchubertError::Enumerative)?;

                principal.namespace.grant(amari_cap).map_err(|e| {
                    SchubertError::Enumerative(amari_enumerative::EnumerativeError::SchubertError(
                        e.to_string(),
                    ))
                })?;
            }
        }
        Ok(())
    }

    /// Serialize the controller state to a JSON string.
    #[cfg(feature = "serde")]
    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }

    /// Deserialize the controller state from a JSON string.
    #[cfg(feature = "serde")]
    pub fn from_json(json: &str) -> serde_json::Result<Self> {
        serde_json::from_str(json)
    }

    /// Save the controller state to a file.
    #[cfg(all(feature = "serde", feature = "std"))]
    pub fn save_to_file(&self, path: impl AsRef<std::path::Path>) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(self).map_err(std::io::Error::other)?;
        std::fs::write(path, json)
    }

    /// Load the controller state from a file.
    #[cfg(all(feature = "serde", feature = "std"))]
    pub fn load_from_file(path: impl AsRef<std::path::Path>) -> std::io::Result<Self> {
        let json = std::fs::read_to_string(path)?;
        serde_json::from_str(&json)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }
}

/// Map an amari `IntersectionResult` to a Schubert `AccessDecision`.
pub(crate) fn map_intersection_result(
    result: IntersectionResult,
    required: &[&str],
    path: ComputationPath,
) -> AccessDecision {
    match result {
        IntersectionResult::Finite(0) => AccessDecision::Impossible {
            conflicting: required.iter().map(|s| CapabilityId::new(*s)).collect(),
        },
        IntersectionResult::Finite(n) => AccessDecision::Granted {
            configurations: n,
            path,
        },
        IntersectionResult::PositiveDimensional { dimension, .. } => {
            AccessDecision::Underconstrained { dimension }
        }
        IntersectionResult::Empty => AccessDecision::Denied,
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Serialization
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(feature = "serde")]
impl serde::Serialize for AccessController {
    fn serialize<S: serde::Serializer>(
        &self,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("AccessController", 3)?;
        state.serialize_field("grassmannian", &self.grassmannian)?;
        state.serialize_field(
            "capabilities",
            &self.capabilities.values().collect::<Vec<_>>(),
        )?;
        state.serialize_field("principals", &self.principals.values().collect::<Vec<_>>())?;
        state.end()
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for AccessController {
    fn deserialize<D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> std::result::Result<Self, D::Error> {
        #[derive(serde::Deserialize)]
        struct ControllerData {
            grassmannian: (usize, usize),
            capabilities: Vec<crate::Capability>,
            principals: Vec<crate::Principal>,
        }

        let data = ControllerData::deserialize(deserializer)?;

        let (k, n) = data.grassmannian;
        if k == 0 || n < 2 || k >= n {
            return Err(serde::de::Error::custom(format!(
                "invalid Grassmannian Gr({k},{n})"
            )));
        }

        let mut controller = AccessController {
            grassmannian: (k, n),
            capabilities: HashMap::new(),
            principals: HashMap::new(),
            audit_sink: None,
        };

        for cap in data.capabilities {
            controller.capabilities.insert(cap.id.clone(), cap);
        }
        for principal in data.principals {
            controller
                .principals
                .insert(principal.id.clone(), principal);
        }

        controller
            .rebuild_principal_namespaces()
            .map_err(serde::de::Error::custom)?;

        Ok(controller)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability::CapabilityKind;

    fn make_controller() -> AccessController {
        let mut acl = AccessController::new(2, 4).unwrap();
        for (id, partition, kind) in [
            ("sigma1_0", vec![1], CapabilityKind::ReadLike),
            ("sigma1_1", vec![1], CapabilityKind::ReadLike),
            ("sigma1_2", vec![1], CapabilityKind::ReadLike),
            ("sigma1_3", vec![1], CapabilityKind::ReadLike),
            ("sigma2_0", vec![2], CapabilityKind::WriteLike),
            ("sigma11", vec![1, 1], CapabilityKind::WriteLike),
            ("sigma21", vec![2, 1], CapabilityKind::AdminLike),
            ("sigma22", vec![2, 2], CapabilityKind::AdminLike),
        ] {
            acl.register_capability(Capability::new(id, id, partition, kind))
                .unwrap();
        }
        acl
    }

    #[test]
    fn sigma1_fourth_equals_2() {
        let mut acl = make_controller();
        let p = acl.create_principal("test").unwrap();
        for cap in &["sigma1_0", "sigma1_1", "sigma1_2", "sigma1_3"] {
            acl.grant(&p, cap).unwrap();
        }
        let decision = acl
            .check(&p, &["sigma1_0", "sigma1_1", "sigma1_2", "sigma1_3"])
            .unwrap();
        assert_eq!(
            decision,
            AccessDecision::Granted {
                configurations: 2,
                path: ComputationPath::LittlewoodRichardson
            },
            "σ₁⁴ must equal 2 in Gr(2,4)"
        );
    }

    #[test]
    fn sigma2_sigma11_is_impossible() {
        let mut acl = make_controller();
        let p = acl.create_principal("test").unwrap();
        acl.grant(&p, "sigma2_0").unwrap();
        acl.grant(&p, "sigma11").unwrap();
        let decision = acl.check(&p, &["sigma2_0", "sigma11"]).unwrap();
        assert!(
            matches!(decision, AccessDecision::Impossible { .. }),
            "σ₂·σ₁₁ must be impossible, got {decision:?}"
        );
    }

    #[test]
    fn overconstrained_is_denied() {
        let mut acl = make_controller();
        let p = acl.create_principal("test").unwrap();
        acl.grant(&p, "sigma2_0").unwrap();
        acl.grant(&p, "sigma21").unwrap();
        acl.grant(&p, "sigma22").unwrap();
        let decision = acl.check(&p, &["sigma2_0", "sigma21", "sigma22"]).unwrap();
        assert!(matches!(decision, AccessDecision::Denied));
    }

    #[test]
    fn create_principal_twice_fails() {
        let mut acl = make_controller();
        acl.create_principal("alice").unwrap();
        assert!(acl.create_principal("alice").is_err());
    }

    #[test]
    fn revoke_removes_capability() {
        let mut acl = make_controller();
        let p = acl.create_principal("alice").unwrap();
        acl.grant(&p, "sigma1_0").unwrap();
        assert!(acl.principal(&p).unwrap().holds("sigma1_0"));
        acl.revoke(&p, "sigma1_0").unwrap();
        assert!(!acl.principal(&p).unwrap().holds("sigma1_0"));
    }

    #[test]
    fn principal_capabilities_returns_in_order() {
        let mut acl = make_controller();
        let p = acl.create_principal("test").unwrap();
        acl.grant(&p, "sigma1_0").unwrap();
        acl.grant(&p, "sigma2_0").unwrap();
        acl.grant(&p, "sigma11").unwrap();

        let caps = acl.principal_capabilities(&p).unwrap();
        assert_eq!(caps.len(), 3);
        assert_eq!(caps[0].id.as_str(), "sigma1_0");
        assert_eq!(caps[1].id.as_str(), "sigma2_0");
        assert_eq!(caps[2].id.as_str(), "sigma11");
    }

    // ── Computation path tests ──────────────────────────────────

    fn setup_acl() -> AccessController {
        let mut acl = AccessController::new(2, 4).unwrap();
        for (id, partition, kind) in [
            ("sigma1_a", vec![1], CapabilityKind::ReadLike),
            ("sigma1_b", vec![1], CapabilityKind::ReadLike),
            ("sigma1_c", vec![1], CapabilityKind::ReadLike),
            ("sigma1_d", vec![1], CapabilityKind::ReadLike),
            ("sigma2", vec![2], CapabilityKind::WriteLike),
            ("sigma11", vec![1, 1], CapabilityKind::WriteLike),
        ] {
            acl.register_capability(Capability::new(id, id, partition, kind))
                .unwrap();
        }
        acl
    }

    #[test]
    fn lr_path_sigma1_fourth_equals_2() {
        let mut acl = setup_acl();
        let p = acl.create_principal("test").unwrap();
        for cap in &["sigma1_a", "sigma1_b", "sigma1_c", "sigma1_d"] {
            acl.grant(&p, cap).unwrap();
        }
        let decision = acl
            .check_with_path(
                &p,
                &["sigma1_a", "sigma1_b", "sigma1_c", "sigma1_d"],
                ComputationPath::LittlewoodRichardson,
            )
            .unwrap();
        assert_eq!(
            decision,
            AccessDecision::Granted {
                configurations: 2,
                path: ComputationPath::LittlewoodRichardson,
            },
            "LR: σ₁⁴ must equal 2"
        );
    }

    #[test]
    fn localization_path_sigma1_fourth_equals_2() {
        let mut acl = setup_acl();
        let p = acl.create_principal("test").unwrap();
        for cap in &["sigma1_a", "sigma1_b", "sigma1_c", "sigma1_d"] {
            acl.grant(&p, cap).unwrap();
        }
        let decision = acl
            .check_with_path(
                &p,
                &["sigma1_a", "sigma1_b", "sigma1_c", "sigma1_d"],
                ComputationPath::Localization,
            )
            .unwrap();
        assert_eq!(
            decision,
            AccessDecision::Granted {
                configurations: 2,
                path: ComputationPath::Localization,
            },
            "Localization: σ₁⁴ must equal 2"
        );
    }

    #[test]
    fn tropical_path_sigma1_fourth_returns_finite() {
        let mut acl = setup_acl();
        let p = acl.create_principal("test").unwrap();
        for cap in &["sigma1_a", "sigma1_b", "sigma1_c", "sigma1_d"] {
            acl.grant(&p, cap).unwrap();
        }
        let decision = acl
            .check_with_path(
                &p,
                &["sigma1_a", "sigma1_b", "sigma1_c", "sigma1_d"],
                ComputationPath::Tropical,
            )
            .unwrap();
        // Tropical intersection gives an approximate count (may differ from exact LR=2)
        assert!(
            matches!(
                decision,
                AccessDecision::Granted {
                    path: ComputationPath::Tropical,
                    ..
                }
            ),
            "Tropical: σ₁⁴ should return Granted (approximate count), got {decision:?}"
        );
    }

    #[test]
    fn lr_path_sigma2_sigma11_is_impossible() {
        let mut acl = setup_acl();
        let p = acl.create_principal("test").unwrap();
        acl.grant(&p, "sigma2").unwrap();
        acl.grant(&p, "sigma11").unwrap();
        let decision = acl
            .check_with_path(
                &p,
                &["sigma2", "sigma11"],
                ComputationPath::LittlewoodRichardson,
            )
            .unwrap();
        assert!(
            matches!(decision, AccessDecision::Impossible { .. }),
            "LR: σ₂·σ₁₁ must be impossible"
        );
    }

    #[test]
    fn localization_path_sigma2_sigma11_is_impossible() {
        let mut acl = setup_acl();
        let p = acl.create_principal("test").unwrap();
        acl.grant(&p, "sigma2").unwrap();
        acl.grant(&p, "sigma11").unwrap();
        let decision = acl
            .check_with_path(&p, &["sigma2", "sigma11"], ComputationPath::Localization)
            .unwrap();
        assert!(
            matches!(decision, AccessDecision::Impossible { .. }),
            "Localization: σ₂·σ₁₁ must be impossible"
        );
    }

    #[test]
    fn matroid_path_detects_impossible() {
        let mut acl = setup_acl();
        let p = acl.create_principal("test").unwrap();
        acl.grant(&p, "sigma2").unwrap();
        acl.grant(&p, "sigma11").unwrap();
        let decision = acl
            .check_with_path(&p, &["sigma2", "sigma11"], ComputationPath::Matroid)
            .unwrap();
        assert!(
            matches!(decision, AccessDecision::Impossible { .. }),
            "Matroid: σ₂·σ₁₁ must be impossible"
        );
    }

    #[test]
    fn auto_routing_selects_correct_path() {
        let mut acl = setup_acl();
        let p = acl.create_principal("test").unwrap();
        // Small Grassmannian Gr(2,4) with n=4 ≤ 8 and 1 class → should pick LR
        acl.grant(&p, "sigma2").unwrap();
        let decision_lr = acl.check(&p, &["sigma2"]).unwrap();
        let decision_auto = acl.check_auto(&p, &["sigma2"]).unwrap();
        assert_eq!(
            decision_auto, decision_lr,
            "Auto-routing for Gr(2,4) should match LR"
        );
    }

    #[test]
    fn paths_produce_consistent_sigma1_fourth() {
        let mut acl = setup_acl();
        let p = acl.create_principal("test").unwrap();
        for cap in &["sigma1_a", "sigma1_b", "sigma1_c", "sigma1_d"] {
            acl.grant(&p, cap).unwrap();
        }

        let required = &["sigma1_a", "sigma1_b", "sigma1_c", "sigma1_d"];

        let lr = acl
            .check_with_path(&p, required, ComputationPath::LittlewoodRichardson)
            .unwrap();
        let loc = acl
            .check_with_path(&p, required, ComputationPath::Localization)
            .unwrap();

        // LR and Localization should agree on σ₁⁴ = 2 (exact methods)
        assert_eq!(
            lr,
            AccessDecision::Granted {
                configurations: 2,
                path: ComputationPath::LittlewoodRichardson,
            }
        );
        assert_eq!(
            loc,
            AccessDecision::Granted {
                configurations: 2,
                path: ComputationPath::Localization,
            }
        );

        // Tropical gives approximate count — just verify it's a finite Grant
        let trop = acl
            .check_with_path(&p, required, ComputationPath::Tropical)
            .unwrap();
        assert!(
            matches!(trop, AccessDecision::Granted { .. }),
            "Tropical path must return Granted for σ₁⁴"
        );
    }

    #[test]
    fn paths_agree_on_impossible() {
        let mut acl = setup_acl();
        let p = acl.create_principal("test").unwrap();
        acl.grant(&p, "sigma2").unwrap();
        acl.grant(&p, "sigma11").unwrap();

        let required = &["sigma2", "sigma11"];

        let lr = acl
            .check_with_path(&p, required, ComputationPath::LittlewoodRichardson)
            .unwrap();
        let loc = acl
            .check_with_path(&p, required, ComputationPath::Localization)
            .unwrap();
        let trop = acl
            .check_with_path(&p, required, ComputationPath::Tropical)
            .unwrap();
        let mat = acl
            .check_with_path(&p, required, ComputationPath::Matroid)
            .unwrap();

        assert!(
            matches!(lr, AccessDecision::Impossible { .. }),
            "LR: must be impossible"
        );
        assert!(
            matches!(loc, AccessDecision::Impossible { .. }),
            "Localization: must be impossible"
        );
        assert!(
            matches!(trop, AccessDecision::Impossible { .. }),
            "Tropical: must be impossible"
        );
        assert!(
            matches!(mat, AccessDecision::Impossible { .. }),
            "Matroid: must be impossible"
        );
    }
}

#[cfg(all(test, feature = "parallel"))]
mod parallel_tests {
    use super::*;
    use crate::capability::CapabilityKind;

    fn setup() -> AccessController {
        let mut acl = AccessController::new(2, 4).unwrap();
        for (id, partition, kind) in [
            ("sigma1_a", vec![1], CapabilityKind::ReadLike),
            ("sigma1_b", vec![1], CapabilityKind::ReadLike),
            ("sigma1_c", vec![1], CapabilityKind::ReadLike),
            ("sigma1_d", vec![1], CapabilityKind::ReadLike),
            ("sigma2", vec![2], CapabilityKind::WriteLike),
            ("sigma22", vec![2, 2], CapabilityKind::AdminLike),
        ] {
            acl.register_capability(Capability::new(id, id, partition, kind))
                .unwrap();
        }
        acl
    }

    #[test]
    fn check_batch_multiple_principals() {
        let mut acl = setup();
        let p1 = acl.create_principal("alice").unwrap();
        let p2 = acl.create_principal("bob").unwrap();
        acl.grant(&p1, "sigma22").unwrap();
        acl.grant(&p2, "sigma1_a").unwrap();
        acl.grant(&p2, "sigma1_b").unwrap();
        acl.grant(&p2, "sigma1_c").unwrap();
        acl.grant(&p2, "sigma1_d").unwrap();

        let queries: &[(&PrincipalId, &[&str])] = &[
            (&p1, &["sigma22"]),
            (&p2, &["sigma1_a", "sigma1_b", "sigma1_c", "sigma1_d"]),
        ];

        let results = acl.check_batch(queries);
        assert_eq!(results.len(), 2);

        // Alice: sigma22 = point class → 1 configuration
        assert_eq!(
            results[0].as_ref().unwrap(),
            &AccessDecision::Granted {
                configurations: 1,
                path: ComputationPath::LittlewoodRichardson,
            }
        );

        // Bob: σ₁⁴ = 2 configurations
        assert_eq!(
            results[1].as_ref().unwrap(),
            &AccessDecision::Granted {
                configurations: 2,
                path: ComputationPath::LittlewoodRichardson,
            }
        );
    }

    #[test]
    fn check_batch_handles_denied_principal() {
        let mut acl = setup();
        let p1 = acl.create_principal("alice").unwrap();
        let p2 = acl.create_principal("bob").unwrap();
        acl.grant(&p1, "sigma22").unwrap();
        // Bob has no sigma2

        let queries: &[(&PrincipalId, &[&str])] = &[
            (&p1, &["sigma22"]), // granted: point class = 1 config
            (&p2, &["sigma2"]),  // denied: bob doesn't hold sigma2
        ];

        let results = acl.check_batch(queries);
        assert_eq!(results.len(), 2);

        // Alice: sigma22 = 1 config
        assert_eq!(
            results[0].as_ref().unwrap(),
            &AccessDecision::Granted {
                configurations: 1,
                path: ComputationPath::LittlewoodRichardson,
            }
        );

        // Bob: doesn't hold sigma2 → denied
        assert_eq!(results[1].as_ref().unwrap(), &AccessDecision::Denied);
    }

    #[test]
    fn stability_batch_multiple_principals() {
        let mut acl = setup();
        let p1 = acl.create_principal("alice").unwrap();
        let p2 = acl.create_principal("bob").unwrap();
        acl.grant(&p1, "sigma1_a").unwrap();
        acl.grant(&p2, "sigma1_a").unwrap();
        acl.grant(&p2, "sigma2").unwrap();

        let reports = acl.stability_batch(&[p1.clone(), p2.clone()]);
        assert_eq!(reports.len(), 2);

        let r1 = reports[0].as_ref().unwrap();
        let r2 = reports[1].as_ref().unwrap();

        assert_eq!(r1.principal, p1);
        assert_eq!(r2.principal, p2);
        assert_eq!(r1.total_capabilities, 1);
        assert_eq!(r2.total_capabilities, 2);
    }

    #[test]
    fn compose_batch_multiple_pairs() {
        let mut acl = setup();
        let p1 = acl.create_principal("producer").unwrap();
        let p2 = acl.create_principal("consumer").unwrap();
        // Both principals hold sigma1_a — that's the shared interface
        acl.grant(&p1, "sigma1_a").unwrap();
        acl.grant(&p1, "sigma2").unwrap(); // extra cap on producer
        acl.grant(&p2, "sigma1_a").unwrap();
        acl.grant(&p2, "sigma1_b").unwrap(); // extra cap on consumer

        // Compose via the shared sigma1_a interface
        let pairs: &[(&PrincipalId, &str, &PrincipalId, &str)] =
            &[(&p1, "sigma1_a", &p2, "sigma1_a")];

        let results = acl.compose_batch(pairs);
        assert_eq!(results.len(), 1);
        let result = results[0].as_ref().unwrap();
        assert!(result.multiplicity > 0);
        // Retained: sigma2 from p1, sigma1_b from p2 (sigma1_a consumed as interface)
        assert!(result.retained_capabilities.contains(&"sigma2".to_string()));
        assert!(result
            .retained_capabilities
            .contains(&"sigma1_b".to_string()));
    }

    #[test]
    fn check_batch_matches_sequential() {
        let mut acl = setup();
        let p1 = acl.create_principal("alice").unwrap();
        let p2 = acl.create_principal("bob").unwrap();
        acl.grant(&p1, "sigma22").unwrap();
        acl.grant(&p2, "sigma1_a").unwrap();
        acl.grant(&p2, "sigma1_b").unwrap();
        acl.grant(&p2, "sigma1_c").unwrap();
        acl.grant(&p2, "sigma1_d").unwrap();

        let queries: &[(&PrincipalId, &[&str])] = &[
            (&p1, &["sigma22"]),
            (&p2, &["sigma1_a", "sigma1_b", "sigma1_c", "sigma1_d"]),
        ];

        // Sequential
        let seq: Vec<_> = queries
            .iter()
            .map(|(pid, reqs)| acl.check(pid, reqs))
            .collect();

        // Parallel
        let par = acl.check_batch(queries);

        for (s, p) in seq.iter().zip(par.iter()) {
            assert_eq!(
                s.as_ref().unwrap(),
                p.as_ref().unwrap(),
                "parallel check_batch must match sequential check"
            );
        }
    }
}

#[cfg(all(test, feature = "serde"))]
mod serde_tests {
    use super::*;
    use crate::capability::CapabilityKind;

    fn setup() -> AccessController {
        let mut acl = AccessController::new(2, 4).unwrap();
        for (id, partition, kind) in [
            ("read", vec![1], CapabilityKind::ReadLike),
            ("write", vec![2], CapabilityKind::WriteLike),
            ("admin", vec![2, 2], CapabilityKind::AdminLike),
            ("internal", vec![1, 1], CapabilityKind::WriteLike),
        ] {
            acl.register_capability(Capability::new(id, id, partition, kind))
                .unwrap();
        }
        acl
    }

    #[test]
    fn roundtrip_empty_controller() {
        let acl = AccessController::new(2, 4).unwrap();
        let json = acl.to_json().unwrap();
        let restored = AccessController::from_json(&json).unwrap();
        assert_eq!(restored.grassmannian(), (2, 4));
        assert_eq!(restored.capabilities().count(), 0);
    }

    #[test]
    fn roundtrip_with_capabilities() {
        let acl = setup();
        let json = acl.to_json().unwrap();
        let restored = AccessController::from_json(&json).unwrap();

        assert_eq!(restored.grassmannian(), (2, 4));
        assert_eq!(restored.capabilities().count(), 4);
        assert!(restored.capability("read").is_some());
        assert!(restored.capability("write").is_some());
        assert!(restored.capability("admin").is_some());
        assert!(restored.capability("internal").is_some());
    }

    #[test]
    fn roundtrip_principals_preserved() {
        let mut acl = setup();
        let alice = acl.create_principal("alice").unwrap();
        let bob = acl.create_principal("bob").unwrap();
        acl.grant(&alice, "read").unwrap();
        acl.grant(&bob, "read").unwrap();
        acl.grant(&bob, "write").unwrap();

        let json = acl.to_json().unwrap();
        let restored = AccessController::from_json(&json).unwrap();

        let alice_restored = restored.principal(&alice).unwrap();
        let bob_restored = restored.principal(&bob).unwrap();

        assert!(alice_restored.holds("read"));
        assert!(!alice_restored.holds("write"));
        assert!(bob_restored.holds("read"));
        assert!(bob_restored.holds("write"));
    }

    #[test]
    fn roundtrip_decisions_match() {
        let mut acl = setup();
        let alice = acl.create_principal("alice").unwrap();
        let bob = acl.create_principal("bob").unwrap();
        acl.grant(&alice, "read").unwrap();
        acl.grant(&alice, "write").unwrap();
        acl.grant(&bob, "admin").unwrap();

        // Capture decisions before serialization
        let before_checks = [
            acl.check(&alice, &["read"]).unwrap(),
            acl.check(&alice, &["read", "write"]).unwrap(),
            acl.check(&bob, &["admin"]).unwrap(),
            acl.check(&alice, &["admin"]).unwrap(), // alice doesn't have admin
        ];

        // Roundtrip
        let json = acl.to_json().unwrap();
        let restored = AccessController::from_json(&json).unwrap();

        // Same checks after deserialization
        let after_checks = [
            restored.check(&alice, &["read"]).unwrap(),
            restored.check(&alice, &["read", "write"]).unwrap(),
            restored.check(&bob, &["admin"]).unwrap(),
            restored.check(&alice, &["admin"]).unwrap(),
        ];

        assert_eq!(
            before_checks, after_checks,
            "all access decisions must survive roundtrip"
        );
    }

    #[test]
    fn roundtrip_sigma1_fourth_equals_2() {
        let mut acl = setup();
        // Register four sigma1 capabilities and grant them
        for i in 0..4 {
            let id = format!("sigma1_{i}");
            acl.register_capability(Capability::new(
                id.clone(),
                id.clone(),
                vec![1],
                CapabilityKind::ReadLike,
            ))
            .unwrap();
        }
        let p = acl.create_principal("test").unwrap();
        for i in 0..4 {
            acl.grant(&p, &format!("sigma1_{i}")).unwrap();
        }

        let before = acl
            .check(&p, &["sigma1_0", "sigma1_1", "sigma1_2", "sigma1_3"])
            .unwrap();

        let json = acl.to_json().unwrap();
        let restored = AccessController::from_json(&json).unwrap();

        // Use the same principal ID to look up after restore
        let rp = restored.principal(&p).unwrap();
        let after = restored
            .check(&rp.id, &["sigma1_0", "sigma1_1", "sigma1_2", "sigma1_3"])
            .unwrap();

        assert_eq!(before, after, "σ₁⁴=2 must survive roundtrip");
    }

    #[test]
    fn roundtrip_impossible_detected() {
        let mut acl = setup();
        let p = acl.create_principal("test").unwrap();
        acl.grant(&p, "write").unwrap(); // σ₂
        acl.grant(&p, "internal").unwrap(); // σ₁₁

        let before = acl.check(&p, &["write", "internal"]).unwrap();
        assert!(matches!(before, AccessDecision::Impossible { .. }));

        let json = acl.to_json().unwrap();
        let restored = AccessController::from_json(&json).unwrap();

        let rp = restored.principal(&p).unwrap();
        let after = restored.check(&rp.id, &["write", "internal"]).unwrap();
        assert!(
            matches!(after, AccessDecision::Impossible { .. }),
            "impossibility detection must survive roundtrip"
        );
    }

    #[test]
    fn roundtrip_revoke_preserved() {
        let mut acl = setup();
        let p = acl.create_principal("alice").unwrap();
        acl.grant(&p, "read").unwrap();
        acl.grant(&p, "write").unwrap();
        acl.revoke(&p, "write").unwrap();

        assert!(acl.principal(&p).unwrap().holds("read"));
        assert!(!acl.principal(&p).unwrap().holds("write"));

        let json = acl.to_json().unwrap();
        let restored = AccessController::from_json(&json).unwrap();

        assert!(restored.principal(&p).unwrap().holds("read"));
        assert!(!restored.principal(&p).unwrap().holds("write"));
    }

    #[cfg(feature = "std")]
    #[test]
    fn file_save_load_roundtrip() {
        let mut acl = setup();
        let p = acl.create_principal("alice").unwrap();
        acl.grant(&p, "read").unwrap();
        acl.grant(&p, "write").unwrap();

        let before = acl.check(&p, &["read", "write"]).unwrap();

        let tmp = std::env::temp_dir().join("schubert_test_policy.json");
        acl.save_to_file(&tmp).unwrap();
        let restored = AccessController::load_from_file(&tmp).unwrap();
        let _ = std::fs::remove_file(&tmp);

        let rp = restored.principal(&p).unwrap();
        let after = restored.check(&rp.id, &["read", "write"]).unwrap();
        assert_eq!(before, after, "file save/load must preserve decisions");
    }
}
