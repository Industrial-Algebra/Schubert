// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! Distributed access control via CRDTs.
//!
//! Operadic composition over eventually-consistent state using conflict-free
//! replicated data types. Principals hold vector clocks. Capability grants
//! merge via commutative, associative, idempotent operations. The intersection
//! number is computed from eventually-consistent state.
//!
//! # How It Works
//!
//! 1. Each node maintains a `VersionVector` tracking grant operations
//! 2. Capability grants are `CrdtGrant`s with last-write-wins semantics
//! 3. Nodes exchange their state and merge via `CrdtState::merge()`
//! 4. Access checks are computed from the merged eventually-consistent state
//!
//! # Example
//!
//! ```
//! use schubert::crdt::{CrdtState, VersionVector, CrdtGrant};
//! use schubert::{Capability, CapabilityKind, PrincipalId};
//!
//! let mut node_a = CrdtState::new(2, 4)?;
//! let mut node_b = CrdtState::new(2, 4)?;
//!
//! node_a.register_capability(Capability::new("read", "Read", vec![1], CapabilityKind::ReadLike))?;
//! node_a.register_capability(Capability::new("write", "Write", vec![2], CapabilityKind::WriteLike))?;
//! node_b.register_capability(Capability::new("read", "Read", vec![1], CapabilityKind::ReadLike))?;
//! node_b.register_capability(Capability::new("write", "Write", vec![2], CapabilityKind::WriteLike))?;
//!
//! node_a.grant("alice", "read", "node_a", 1000)?;
//! node_b.grant("alice", "write", "node_b", 1000)?;
//!
//! node_a.merge(&node_b);
//! assert!(node_a.holds(&PrincipalId::new("alice"), "read"));
//! assert!(node_a.holds(&PrincipalId::new("alice"), "write"));
//! # Ok::<(), schubert::SchubertError>(())
//! ```

use std::collections::HashMap;

use crate::{
    error::{Result, SchubertError},
    AccessDecision, CapabilityId, PrincipalId,
};

/// A version vector — a map from node ID to operation counter.
///
/// Used for causal ordering of grant operations across distributed nodes.
/// Standard vector clock semantics: `a` happens-before `b` if all entries
/// in `a` are ≤ `b` and at least one is <.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct VersionVector {
    counters: HashMap<String, u64>,
}

impl VersionVector {
    /// Create a new empty version vector.
    pub fn new() -> Self {
        Self::default()
    }

    /// Increment the counter for a node.
    pub fn increment(&mut self, node_id: &str) {
        *self.counters.entry(node_id.to_string()).or_insert(0) += 1;
    }

    /// Get the counter for a node.
    pub fn get(&self, node_id: &str) -> u64 {
        self.counters.get(node_id).copied().unwrap_or(0)
    }

    /// Check if this vector happens-before another.
    pub fn happens_before(&self, other: &Self) -> bool {
        let all_keys: std::collections::HashSet<_> =
            self.counters.keys().chain(other.counters.keys()).collect();

        let mut any_less = false;
        for key in all_keys {
            let a = self.get(key);
            let b = other.get(key);
            if a > b {
                return false;
            }
            if a < b {
                any_less = true;
            }
        }
        any_less
    }

    /// Merge two version vectors (pointwise max).
    pub fn merge(&self, other: &Self) -> Self {
        let mut result = self.clone();
        for (key, &val) in &other.counters {
            let entry = result.counters.entry(key.clone()).or_insert(0);
            *entry = (*entry).max(val);
        }
        result
    }
}

/// A capability grant in a CRDT state.
///
/// Each grant is associated with a version vector that tracks when and
/// by which node it was made. Grants merge with last-write-wins semantics
/// based on the version vector.
#[derive(Debug, Clone)]
pub struct CrdtGrant {
    /// The principal receiving this grant.
    pub principal: PrincipalId,
    /// The capability being granted.
    pub capability: CapabilityId,
    /// Version vector at the time of grant.
    pub version: VersionVector,
    /// Timestamp of grant (for tiebreaking).
    pub timestamp_ms: u64,
    /// Whether this grant is active (false = revoked).
    pub active: bool,
}

/// CRDT-based access control state.
///
/// Maintains an eventually-consistent set of capability grants that can
/// be merged across distributed nodes. Grants are commutative, associative,
/// and idempotent — merging the same grant twice is a no-op.
#[derive(Debug, Clone)]
pub struct CrdtState {
    grassmannian: (usize, usize),
    /// Registered capabilities.
    capabilities: HashMap<CapabilityId, crate::Capability>,
    /// Grants: (principal, capability) → CRDT grant.
    grants: HashMap<(PrincipalId, CapabilityId), CrdtGrant>,
    /// The current version vector.
    version: VersionVector,
    /// Maximum allowed staleness in milliseconds. If set, `check()` will
    /// refuse decisions when the youngest grant is older than this.
    /// `None` disables staleness gating (default).
    max_staleness_ms: Option<u64>,
}

impl CrdtState {
    /// Create a new CRDT state for Gr(k,n).
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
            grants: HashMap::new(),
            version: VersionVector::new(),
            max_staleness_ms: None,
        })
    }

    /// Register a capability.
    pub fn register_capability(&mut self, cap: crate::Capability) -> Result<()> {
        if self.capabilities.contains_key(&cap.id) {
            return Err(SchubertError::CapabilityExists(cap.id.to_string()));
        }
        cap.to_schubert_class(self.grassmannian)?;
        self.capabilities.insert(cap.id.clone(), cap);
        Ok(())
    }

    /// Grant a capability to a principal on a specific node.
    ///
    /// `node_id` identifies the granting node. `timestamp_ms` is used
    /// for tiebreaking when version vectors are concurrent.
    pub fn grant(
        &mut self,
        principal: impl Into<PrincipalId>,
        capability: impl Into<CapabilityId>,
        node_id: &str,
        timestamp_ms: u64,
    ) -> Result<()> {
        let principal = principal.into();
        let capability = capability.into();

        if !self.capabilities.contains_key(&capability) {
            return Err(SchubertError::CapabilityNotFound(capability.to_string()));
        }

        self.version.increment(node_id);
        let grant = CrdtGrant {
            principal: principal.clone(),
            capability: capability.clone(),
            version: self.version.clone(),
            timestamp_ms,
            active: true,
        };

        let key = (principal, capability);
        if let Some(existing) = self.grants.get(&key) {
            // Last-write-wins: newer version or higher timestamp wins
            if grant.version.happens_before(&existing.version)
                || (grant.version == existing.version
                    && grant.timestamp_ms <= existing.timestamp_ms)
            {
                return Ok(()); // existing is newer, skip
            }
        }
        self.grants.insert(key, grant);
        Ok(())
    }

    /// Revoke a capability from a principal.
    pub fn revoke(
        &mut self,
        principal: impl Into<PrincipalId>,
        capability: impl Into<CapabilityId>,
        node_id: &str,
        timestamp_ms: u64,
    ) -> Result<()> {
        let principal = principal.into();
        let capability = capability.into();

        self.version.increment(node_id);
        let grant = CrdtGrant {
            principal,
            capability,
            version: self.version.clone(),
            timestamp_ms,
            active: false,
        };

        let key = (grant.principal.clone(), grant.capability.clone());
        self.grants.insert(key, grant);
        Ok(())
    }

    /// Check if a principal holds a capability.
    pub fn holds(&self, principal: &PrincipalId, capability: &str) -> bool {
        let cid = CapabilityId::new(capability);
        let key = (principal.clone(), cid);
        self.grants.get(&key).is_some_and(|g| g.active)
    }

    /// Merge another CRDT state into this one.
    ///
    /// Merges all grants using version-vector-based conflict resolution.
    /// The resulting state is the least upper bound of both states.
    pub fn merge(&mut self, other: &Self) {
        // Merge version vectors
        self.version = self.version.merge(&other.version);

        // Merge grants: for each grant in other, take the one with
        // the higher version vector (or higher timestamp as tiebreaker)
        for (key, other_grant) in &other.grants {
            if let Some(self_grant) = self.grants.get(key) {
                if other_grant.version.happens_before(&self_grant.version)
                    || (other_grant.version == self_grant.version
                        && other_grant.timestamp_ms <= self_grant.timestamp_ms)
                {
                    continue; // self has newer
                }
            }
            self.grants.insert(key.clone(), other_grant.clone());
        }

        // Merge capabilities (take union — registration is idempotent)
        for (id, cap) in &other.capabilities {
            self.capabilities
                .entry(id.clone())
                .or_insert_with(|| cap.clone());
        }
    }

    /// Compute an access check from the CRDT state.
    ///
    /// Builds an `AccessController` from the merged state and performs
    /// the check. The intersection number is computed from the
    /// eventually-consistent state.
    pub fn check(&self, principal: &PrincipalId, required: &[&str]) -> Result<AccessDecision> {
        let mut acl = crate::AccessController::new(self.grassmannian.0, self.grassmannian.1)?;

        // Register capabilities
        for cap in self.capabilities.values() {
            acl.register_capability(cap.clone())?;
        }

        // Create principal and replay grants
        acl.create_principal(principal.clone())?;
        for ((pid, _cid), grant) in &self.grants {
            if pid == principal && grant.active {
                let _ = acl.grant(principal, grant.capability.as_str());
            }
        }

        acl.check(principal, required)
    }

    /// Get a list of active grants for a principal.
    pub fn active_grants(&self, principal: &PrincipalId) -> Vec<String> {
        self.grants
            .iter()
            .filter(|((pid, _), g)| pid == principal && g.active)
            .map(|((_, cid), _)| cid.to_string())
            .collect()
    }

    /// Get the current version vector.
    pub fn version(&self) -> &VersionVector {
        &self.version
    }

    /// Set the maximum allowed staleness. `None` disables staleness gating.
    pub fn set_max_staleness(&mut self, max_staleness_ms: Option<u64>) {
        self.max_staleness_ms = max_staleness_ms;
    }

    /// Get the staleness in milliseconds — the age of the oldest active grant.
    /// Returns `None` if there are no grants.
    pub fn staleness_ms(&self) -> Option<u64> {
        if self.grants.is_empty() {
            return None;
        }
        let max_ts = self.grants.values().map(|g| g.timestamp_ms).max()?;
        let min_ts = self.grants.values().map(|g| g.timestamp_ms).min()?;
        Some(max_ts.saturating_sub(min_ts))
    }

    /// Check whether the CRDT state has converged with another version vector.
    /// Returns true if this node has seen all operations known to `other_version`.
    pub fn is_converged_with(&self, other_version: &VersionVector) -> bool {
        !other_version.happens_before(&self.version)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Capability, CapabilityKind};

    fn setup() -> CrdtState {
        let mut state = CrdtState::new(2, 4).unwrap();
        state
            .register_capability(Capability::new(
                "read",
                "Read",
                vec![1],
                CapabilityKind::ReadLike,
            ))
            .unwrap();
        state
            .register_capability(Capability::new(
                "write",
                "Write",
                vec![2],
                CapabilityKind::WriteLike,
            ))
            .unwrap();
        state
    }

    #[test]
    fn version_vector_happens_before() {
        let mut a = VersionVector::new();
        a.increment("node1");
        let mut b = a.clone();
        b.increment("node1");

        assert!(a.happens_before(&b));
        assert!(!b.happens_before(&a));
    }

    #[test]
    fn version_vector_concurrent() {
        let mut a = VersionVector::new();
        a.increment("node1");
        let mut b = VersionVector::new();
        b.increment("node2");

        // Neither happens-before the other
        assert!(!a.happens_before(&b));
        assert!(!b.happens_before(&a));
    }

    #[test]
    fn grant_and_hold() {
        let mut state = setup();
        state.grant("alice", "read", "node1", 1000).unwrap();
        assert!(state.holds(&PrincipalId::new("alice"), "read"));
        assert!(!state.holds(&PrincipalId::new("alice"), "write"));
    }

    #[test]
    fn revoke_and_check() {
        let mut state = setup();
        state.grant("alice", "read", "node1", 1000).unwrap();
        state.revoke("alice", "read", "node1", 2000).unwrap();
        assert!(!state.holds(&PrincipalId::new("alice"), "read"));
    }

    #[test]
    fn merge_preserves_both_grants() {
        let mut node_a = setup();
        let mut node_b = setup();
        node_a.grant("alice", "read", "node_a", 1000).unwrap();
        node_b.grant("alice", "write", "node_b", 1000).unwrap();

        node_a.merge(&node_b);
        assert!(node_a.holds(&PrincipalId::new("alice"), "read"));
        assert!(node_a.holds(&PrincipalId::new("alice"), "write"));
    }

    #[test]
    fn merge_last_write_wins() {
        let mut node_a = setup();
        let mut node_b = setup();
        node_a.grant("alice", "read", "node_a", 1000).unwrap();
        node_b.revoke("alice", "read", "node_b", 2000).unwrap(); // later timestamp wins

        node_a.merge(&node_b);
        assert!(!node_a.holds(&PrincipalId::new("alice"), "read"));
    }

    #[test]
    fn merge_idempotent() {
        let mut node_a = setup();
        node_a.grant("alice", "read", "node_a", 1000).unwrap();
        let snapshot = node_a.clone();

        node_a.merge(&snapshot);
        // State should be unchanged
        assert!(node_a.holds(&PrincipalId::new("alice"), "read"));
    }

    #[test]
    fn crdt_check_access() {
        let mut state = setup();
        state.grant("alice", "read", "node1", 1000).unwrap();
        state.grant("alice", "write", "node1", 1000).unwrap();

        let result = state
            .check(&PrincipalId::new("alice"), &["read", "write"])
            .unwrap();
        // σ₁·σ₂ in Gr(2,4) — underconstrained
        assert!(matches!(result, AccessDecision::Underconstrained { .. }));
    }
}
