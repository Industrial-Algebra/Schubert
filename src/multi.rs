// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: AGPL-3.0-only

//! Multi-Grassmannian access control.
//!
//! A [`MultiController`] manages multiple [`AccessController`] instances,
//! each operating in a different Grassmannian Gr(k,n). This enables
//! cross-domain access control where a principal in one Grassmannian
//! needs to access resources in another.
//!
//! # Cross-Domain Access
//!
//! The simplest form of cross-domain access is **partition validation**:
//! a capability from Gr(k₁,n₁) is checked for validity in Gr(k₂,n₂).
//! If the partition fits within the target Grassmannian's k₂×(n₂−k₂) box,
//! the capability can be translated and checked.
//!
//! # Example
//!
//! ```
//! use schubert::{AccessController, Capability, CapabilityKind, MultiController, PrincipalId};
//!
//! let mut mc = MultiController::new();
//! let gr24 = mc.add_domain(2, 4)?;
//! mc.add_domain(3, 6)?;
//!
//! // Register a capability in Gr(2,4)
//! mc.register_in_domain(
//!     Capability::new("read", "Read", vec![1], CapabilityKind::ReadLike),
//!     &gr24,
//! )?;
//!
//! // Create a principal and grant
//! let alice_id = mc.create_principal("alice", &gr24)?;
//! mc.grant_in_domain(&alice_id, "read", &gr24)?;
//!
//! // Check within same domain
//! mc.check_in_domain(&alice_id, &["read"], &gr24)?;
//! # Ok::<(), schubert::SchubertError>(())
//! ```

use std::collections::HashMap;

use crate::{
    AccessController, AccessDecision, Capability, CapabilityId, PrincipalId, Result, SchubertError,
};

/// Manages multiple access control domains, each in a different Grassmannian.
///
/// Domains are identified by a label (e.g., `"gr4_2"` for Gr(2,4)).
/// Principals, capabilities, and grants are scoped to specific domains.
/// Cross-domain checks validate whether capabilities from one domain
/// can apply in another.
#[derive(Debug, Default)]
pub struct MultiController {
    /// Registered domains: label → AccessController.
    domains: HashMap<String, AccessController>,
    /// Domain labels for each Grassmannian: "gr{k}_{n}" → label.
    grassmannian_labels: HashMap<(usize, usize), String>,
}

impl MultiController {
    /// Create a new empty multi-controller.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a domain for Gr(k,n) with an auto-generated label.
    ///
    /// Labels are generated as `"gr{k}_{n}"`. Use
    /// [`add_domain_named`](Self::add_domain_named) for custom labels.
    pub fn add_domain(&mut self, k: usize, n: usize) -> Result<String> {
        let label = format!("gr{n}_{k}");
        self.add_domain_named(k, n, &label)?;
        Ok(label)
    }

    /// Add a domain for Gr(k,n) with a custom label.
    pub fn add_domain_named(&mut self, k: usize, n: usize, label: &str) -> Result<()> {
        if self.domains.contains_key(label) {
            return Err(SchubertError::Generic(format!(
                "domain '{label}' already exists"
            )));
        }
        let controller = AccessController::new(k, n)?;
        self.domains.insert(label.to_string(), controller);
        self.grassmannian_labels.insert((k, n), label.to_string());
        Ok(())
    }

    /// Get a reference to a domain's controller.
    pub fn domain(&self, label: &str) -> Option<&AccessController> {
        self.domains.get(label)
    }

    /// Get a mutable reference to a domain's controller.
    pub fn domain_mut(&mut self, label: &str) -> Option<&mut AccessController> {
        self.domains.get_mut(label)
    }

    /// List all domain labels.
    pub fn domain_labels(&self) -> Vec<&str> {
        self.domains.keys().map(|s| s.as_str()).collect()
    }

    /// Get the domain label for a Grassmannian.
    pub fn label_for(&self, k: usize, n: usize) -> Option<&str> {
        self.grassmannian_labels.get(&(k, n)).map(|s| s.as_str())
    }

    /// Find a domain that can accept a given partition.
    ///
    /// Returns labels of all domains where the partition fits within
    /// the k×(n−k) box.
    pub fn domains_for_partition(&self, partition: &[usize]) -> Vec<String> {
        let codim: usize = partition.iter().sum();
        let parts = partition.len();

        self.domains
            .iter()
            .filter(|(_, acl)| {
                let (k, n) = acl.grassmannian();
                let dim = k * (n - k);
                parts <= k && partition.first().map_or(true, |&f| f <= n - k) && codim <= dim
            })
            .map(|(label, _)| label.clone())
            .collect()
    }

    // ── Principal operations ──────────────────────────────────────

    /// Create a principal in a specific domain.
    pub fn create_principal(
        &mut self,
        id: impl Into<PrincipalId>,
        domain_label: &str,
    ) -> Result<PrincipalId> {
        let controller = self
            .domains
            .get_mut(domain_label)
            .ok_or_else(|| SchubertError::Generic(format!("domain '{domain_label}' not found")))?;
        controller.create_principal(id)
    }

    /// Grant a capability to a principal in a specific domain.
    pub fn grant_in_domain(
        &mut self,
        principal_id: &PrincipalId,
        capability_id: &str,
        domain_label: &str,
    ) -> Result<()> {
        let controller = self
            .domains
            .get_mut(domain_label)
            .ok_or_else(|| SchubertError::Generic(format!("domain '{domain_label}' not found")))?;
        controller.grant(principal_id, capability_id)
    }

    /// Register a capability in a specific domain.
    pub fn register_in_domain(&mut self, cap: Capability, domain_label: &str) -> Result<()> {
        let controller = self
            .domains
            .get_mut(domain_label)
            .ok_or_else(|| SchubertError::Generic(format!("domain '{domain_label}' not found")))?;
        controller.register_capability(cap)
    }

    /// Check access within a single domain.
    pub fn check_in_domain(
        &self,
        principal_id: &PrincipalId,
        required: &[&str],
        domain_label: &str,
    ) -> Result<AccessDecision> {
        let controller = self
            .domains
            .get(domain_label)
            .ok_or_else(|| SchubertError::Generic(format!("domain '{domain_label}' not found")))?;
        controller.check(principal_id, required)
    }

    // ── Cross-domain operations ───────────────────────────────────

    /// Check whether a principal from one domain can access resources in another.
    ///
    /// This performs **partition validation**: each required capability's
    /// partition is checked against the target Grassmannian. If the
    /// partition fits, it's valid for cross-domain access. If not, the
    /// capability cannot be translated.
    ///
    /// Returns `Denied` if any capability's partition doesn't fit in
    /// the target domain.
    pub fn check_cross_domain(
        &self,
        principal_id: &PrincipalId,
        required: &[&str],
        source_domain: &str,
        target_domain: &str,
    ) -> Result<AccessDecision> {
        let source = self.domains.get(source_domain).ok_or_else(|| {
            SchubertError::Generic(format!("source domain '{source_domain}' not found"))
        })?;
        let target = self.domains.get(target_domain).ok_or_else(|| {
            SchubertError::Generic(format!("target domain '{target_domain}' not found"))
        })?;

        // Verify the principal exists in the source domain
        let principal = source
            .principal(principal_id)
            .ok_or_else(|| SchubertError::PrincipalNotFound(principal_id.to_string()))?;

        // Check if the principal holds all required capabilities
        for cap_id in required {
            if !principal.holds(cap_id) {
                return Ok(AccessDecision::Denied);
            }
        }

        // Translate capabilities to the target Grassmannian
        let (tk, tn) = target.grassmannian();
        let mut translated_classes = Vec::with_capacity(required.len());
        for cap_id_str in required {
            let cid = CapabilityId::new(*cap_id_str);
            let cap = source
                .capability(cap_id_str)
                .ok_or_else(|| SchubertError::CapabilityNotFound(cid.to_string()))?;

            // Check partition fits in target Grassmannian
            let dim = tk * (tn - tk);
            let codim: usize = cap.partition.iter().sum();
            if cap.partition.len() > tk
                || cap.partition.first().map_or(true, |&f| f > tn - tk)
                || codim > dim
            {
                return Ok(AccessDecision::Denied);
            }

            // Build a Schubert class for the target Grassmannian
            let class = cap.to_schubert_class(target.grassmannian())?;
            translated_classes.push(class);
        }

        // Compute intersection in the target Grassmannian
        use crate::ComputationPath;
        let mut all = vec![
            amari_enumerative::SchubertClass::new(vec![], target.grassmannian()).map_err(|e| {
                SchubertError::Enumerative(amari_enumerative::EnumerativeError::SchubertError(
                    e.to_string(),
                ))
            })?,
        ];
        all.extend(translated_classes);

        let mut calc = amari_enumerative::SchubertCalculus::new(target.grassmannian());
        let result = calc.multi_intersect(&all);

        Ok(crate::controller::map_intersection_result(
            result,
            required,
            ComputationPath::LittlewoodRichardson,
        ))
    }

    /// List all capabilities from a source domain that are translatable
    /// to a target domain.
    ///
    /// Returns the IDs of capabilities whose partitions fit within
    /// the target Grassmannian.
    pub fn translatable_capabilities(
        &self,
        source_domain: &str,
        target_domain: &str,
    ) -> Result<Vec<String>> {
        let source = self.domains.get(source_domain).ok_or_else(|| {
            SchubertError::Generic(format!("source domain '{source_domain}' not found"))
        })?;
        let target = self.domains.get(target_domain).ok_or_else(|| {
            SchubertError::Generic(format!("target domain '{target_domain}' not found"))
        })?;

        let (tk, tn) = target.grassmannian();
        let dim = tk * (tn - tk);

        Ok(source
            .capabilities()
            .filter(|cap| {
                let codim: usize = cap.partition.iter().sum();
                cap.partition.len() <= tk
                    && cap.partition.first().map_or(true, |&f| f <= tn - tk)
                    && codim <= dim
            })
            .map(|cap| cap.id.to_string())
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CapabilityKind;

    fn setup() -> MultiController {
        let mut mc = MultiController::new();
        mc.add_domain(2, 4).unwrap(); // gr24: standard RBAC
        mc.add_domain(3, 6).unwrap(); // gr36: multi-tenant

        // Register capabilities in both domains
        mc.register_in_domain(
            Capability::new("read", "Read", vec![1], CapabilityKind::ReadLike),
            "gr4_2",
        )
        .unwrap();
        mc.register_in_domain(
            Capability::new("write", "Write", vec![2], CapabilityKind::WriteLike),
            "gr4_2",
        )
        .unwrap();
        mc.register_in_domain(
            Capability::new("read", "Read", vec![1], CapabilityKind::ReadLike),
            "gr6_3",
        )
        .unwrap();

        mc
    }

    #[test]
    fn add_domains() {
        let mc = setup();
        let labels = mc.domain_labels();
        assert!(labels.contains(&"gr4_2"));
        assert!(labels.contains(&"gr6_3"));
        assert_eq!(mc.domain("gr4_2").unwrap().grassmannian(), (2, 4));
        assert_eq!(mc.domain("gr6_3").unwrap().grassmannian(), (3, 6));
    }

    #[test]
    fn domain_check_same_domain() {
        let mut mc = setup();
        let alice = mc.create_principal("alice", "gr4_2").unwrap();
        mc.grant_in_domain(&alice, "read", "gr4_2").unwrap();

        let decision = mc.check_in_domain(&alice, &["read"], "gr4_2").unwrap();
        assert!(matches!(decision, AccessDecision::Underconstrained { .. }));
    }

    #[test]
    fn cross_domain_translatable() {
        let mut mc = setup();
        let alice = mc.create_principal("alice", "gr4_2").unwrap();
        mc.grant_in_domain(&alice, "read", "gr4_2").unwrap();

        // "read" is σ₁ — fits in both Gr(2,4) and Gr(3,6)
        let translatable = mc.translatable_capabilities("gr4_2", "gr6_3").unwrap();
        assert!(translatable.contains(&"read".to_string()));
    }

    #[test]
    fn cross_domain_check_translatable() {
        let mut mc = setup();
        let alice = mc.create_principal("alice", "gr4_2").unwrap();
        mc.grant_in_domain(&alice, "read", "gr4_2").unwrap();

        // Check cross-domain: reading from gr24 to gr36
        let decision = mc
            .check_cross_domain(&alice, &["read"], "gr4_2", "gr6_3")
            .unwrap();
        // σ₁ in Gr(3,6) — underconstrained (dim 9, codim 1 → dim 8)
        assert!(matches!(decision, AccessDecision::Underconstrained { .. }));
    }

    #[test]
    fn cross_domain_denied_if_not_held() {
        let mut mc = setup();
        let alice = mc.create_principal("alice", "gr4_2").unwrap();
        // alice doesn't hold "write"

        let decision = mc
            .check_cross_domain(&alice, &["write"], "gr4_2", "gr6_3")
            .unwrap();
        assert_eq!(decision, AccessDecision::Denied);
    }

    #[test]
    fn domains_for_partition() {
        let mc = setup();
        let domains = mc.domains_for_partition(&[1]);
        assert!(domains.contains(&"gr4_2".to_string()));
        assert!(domains.contains(&"gr6_3".to_string()));

        // σ₄ doesn't fit in Gr(2,4) — k=2, n-k=2, so max partition part is 2
        let domains = mc.domains_for_partition(&[4]);
        assert!(!domains.contains(&"gr4_2".to_string()));
    }

    #[test]
    fn duplicate_domain_label_rejected() {
        let mut mc = MultiController::new();
        mc.add_domain(2, 4).unwrap();
        assert!(mc.add_domain_named(2, 4, "gr4_2").is_err());
    }
}
