use crate::audit::AuditSink;
use crate::capability::{Capability, CapabilityId};
use crate::decision::{AccessDecision, ComputationPath};
use crate::error::{Result, SchubertError};
use crate::principal::{Principal, PrincipalId};
use amari_enumerative::{IntersectionResult, SchubertCalculus};
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
                k, n,
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
    pub fn grassmannian(&self) -> (usize, usize) { self.grassmannian }

    /// Install an audit sink for recording access decisions.
    ///
    /// The sink receives every decision made by [`check`](Self::check).
    /// Audit failures are silently ignored — they never affect access decisions.
    pub fn set_audit_sink(&mut self, sink: Box<dyn AuditSink>) { self.audit_sink = Some(sink); }

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
            SchubertError::Enumerative(
                amari_enumerative::EnumerativeError::SchubertError(e.to_string()),
            )
        })
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
    /// First verifies the principal holds each required capability, then
    /// computes the Schubert intersection of the principal's namespace
    /// position with the required Schubert classes.
    pub fn check(
        &self,
        principal_id: &PrincipalId,
        required: &[&str],
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

        // Build required Schubert classes from the capability registry
        let mut required_classes = Vec::with_capacity(required.len());
        for cap_id_str in required {
            let cid = CapabilityId::new(*cap_id_str);
            let cap = self
                .capabilities
                .get(&cid)
                .ok_or_else(|| SchubertError::CapabilityNotFound(cid.to_string()))?;
            required_classes.push(cap.to_schubert_class(self.grassmannian)?);
        }

        // Intersect namespace position with required classes
        let mut calc = SchubertCalculus::new(self.grassmannian);
        let mut all = vec![principal.namespace.position.clone()];
        all.extend(required_classes);
        let result = calc.multi_intersect(&all);

        let decision = match result {
            IntersectionResult::Finite(0) => AccessDecision::Impossible {
                conflicting: required.iter().map(|s| CapabilityId::new(*s)).collect(),
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

    /// Check access with an explicit computation path preference.
    pub fn check_with_path(
        &self,
        principal_id: &PrincipalId,
        required: &[&str],
        _preferred_path: ComputationPath,
    ) -> Result<AccessDecision> {
        self.check(principal_id, required)
    }

    /// Return the effective access dimension for a principal.
    pub fn effective_dimension(&self, principal_id: &PrincipalId) -> Result<isize> {
        let p = self
            .principals
            .get(principal_id)
            .ok_or_else(|| SchubertError::PrincipalNotFound(principal_id.to_string()))?;

        let dim = self.grassmannian.0 * (self.grassmannian.1 - self.grassmannian.0);
        let total_codim: usize = p.namespace.capabilities.iter().map(|c| c.codimension()).sum();
        Ok(dim as isize - total_codim as isize)
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
            AccessDecision::Granted { configurations: 2, path: ComputationPath::LittlewoodRichardson },
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
        assert!(matches!(decision, AccessDecision::Impossible { .. }),
            "σ₂·σ₁₁ must be impossible, got {decision:?}");
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
}
