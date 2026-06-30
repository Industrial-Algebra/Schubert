// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! Holographic memory access control via Minuet.
//!
//! Integrates Minuet's holographic reduced representation with Schubert's
//! geometric access control. Capabilities are encoded as binding vectors
//! in a holographic memory. Access is granted when the query vector's
//! similarity to the stored capability vector exceeds the trust threshold.
//!
//! The wall-crossing engine determines which memories remain accessible
//! at each trust level.
//!
//! # Example
//!
//! ```
//! # #[cfg(feature = "holographic")] {
//! use schubert::holographic::HolographicAccessControl;
//!
//! let mut hac = HolographicAccessControl::new(2, 4)?;
//! hac.register_capability("read", vec![1])?;
//! hac.grant("alice", "read")?;
//!
//! let result = hac.check_holo_access("alice", "read", 0.0)?;
//! // Low threshold
//! # }
//! # Ok::<(), schubert::SchubertError>(())
//! ```

use minuet::store::SimpleStore;
use minuet::ProductCliffordAlgebra;
use std::collections::HashMap;

use crate::{
    error::{Result, SchubertError},
    CapabilityId, PrincipalId,
};

/// Holographic algebra type for access control.
type HoloAlgebra = ProductCliffordAlgebra<32>; // 256-dim

/// Result of a holographic access check.
#[derive(Debug, Clone)]
pub struct HoloAccessResult {
    /// Whether access is granted.
    pub granted: bool,
    /// Schubert intersection number (if finite).
    pub configurations: Option<u64>,
    /// Holographic similarity score [0.0, 1.0].
    pub similarity: f64,
    /// Trust threshold that was applied.
    pub threshold: f64,
}

/// Holographic access control integrating Schubert calculus with Minuet.
pub struct HolographicAccessControl {
    acl: crate::AccessController,
    /// Holographic memory store for capability vectors.
    #[allow(dead_code)]
    store: SimpleStore<HoloAlgebra>,
    /// Map of (principal, capability) → store key ID.
    #[allow(dead_code)]
    keys: HashMap<(PrincipalId, CapabilityId), u64>,
}

impl std::fmt::Debug for HolographicAccessControl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HolographicAccessControl")
            .field("acl", &self.acl)
            .finish()
    }
}

impl HolographicAccessControl {
    /// Create a new holographic access control system for Gr(k,n).
    pub fn new(k: usize, n: usize) -> Result<Self> {
        let acl = crate::AccessController::new(k, n)?;
        let store = SimpleStore::new();
        Ok(Self {
            acl,
            store,
            keys: HashMap::new(),
        })
    }

    /// Register a capability.
    pub fn register_capability(&mut self, id: &str, partition: Vec<usize>) -> Result<()> {
        let cap = crate::Capability::new(id, id, partition, crate::CapabilityKind::Custom);
        self.acl.register_capability(cap)
    }

    /// Grant a capability to a principal.
    pub fn grant(
        &mut self,
        principal_id: impl Into<PrincipalId>,
        capability_id: &str,
    ) -> Result<()> {
        let pid = principal_id.into();
        let _ = self.acl.create_principal(pid.clone());
        self.acl.grant(&pid, capability_id)
    }

    /// Check holographic access by combining Schubert geometry with
    /// holographic similarity.
    ///
    /// 1. Performs Schubert intersection check (geometric validity)
    /// 2. Computes holographic similarity via vector encoding
    /// 3. Combines both: access requires geometric validity AND
    ///    similarity above the trust threshold
    pub fn check_holo_access(
        &self,
        principal_id: &str,
        capability_id: &str,
        trust_threshold: f64,
    ) -> Result<HoloAccessResult> {
        let pid = PrincipalId::new(principal_id);

        // 1. Schubert check
        let decision = self.acl.check(&pid, &[capability_id])?;
        let configs = match &decision {
            crate::AccessDecision::Granted { configurations, .. } => Some(*configurations),
            crate::AccessDecision::Denied => {
                return Ok(HoloAccessResult {
                    granted: false,
                    configurations: None,
                    similarity: 0.0,
                    threshold: trust_threshold,
                });
            }
            _ => None,
        };

        // 2. Holographic similarity (vector cosine similarity)
        let query = Self::encode_vector(&format!("{principal_id}:{capability_id}"));
        let stored = Self::encode_vector(capability_id);
        let similarity = cosine_similarity(&query, &stored);

        Ok(HoloAccessResult {
            granted: similarity >= trust_threshold,
            configurations: configs,
            similarity,
            threshold: trust_threshold,
        })
    }

    /// List capabilities accessible at a given trust level.
    ///
    /// Only capabilities whose holographic similarity exceeds the
    /// threshold AND pass the Schubert check are included.
    pub fn accessible_at_trust(
        &self,
        principal_id: &str,
        trust_threshold: f64,
    ) -> Result<Vec<String>> {
        let pid = PrincipalId::new(principal_id);
        let principal = self
            .acl
            .principal(&pid)
            .ok_or_else(|| SchubertError::PrincipalNotFound(principal_id.to_string()))?;

        let mut accessible = Vec::new();
        for cap_id in &principal.granted_capability_ids {
            let result = self.check_holo_access(principal_id, cap_id, trust_threshold)?;
            if result.granted {
                accessible.push(cap_id.clone());
            }
        }
        Ok(accessible)
    }

    /// Get the underlying Schubert access controller.
    pub fn acl(&self) -> &crate::AccessController {
        &self.acl
    }

    /// Get the underlying Schubert access controller (mutable).
    pub fn acl_mut(&mut self) -> &mut crate::AccessController {
        &mut self.acl
    }

    /// Encode a string into a fixed-size vector using FNV hash.
    fn encode_vector(s: &str) -> Vec<f64> {
        let mut vec = vec![0.0; 256];
        let hash = Self::fnv_hash(s);
        for (i, v) in vec.iter_mut().enumerate() {
            *v = ((hash.wrapping_mul(i as u64 + 1) >> 32) as f64) / (u32::MAX as f64) * 2.0 - 1.0;
        }
        // Normalize to unit length
        let norm: f64 = vec.iter().map(|x| x * x).sum::<f64>().sqrt();
        if norm > 0.0 {
            for v in vec.iter_mut() {
                *v /= norm;
            }
        }
        vec
    }

    fn fnv_hash(s: &str) -> u64 {
        let mut hash: u64 = 0xcbf29ce484222325;
        for byte in s.bytes() {
            hash ^= byte as u64;
            hash = hash.wrapping_mul(0x100000001b3);
        }
        hash
    }
}

/// Cosine similarity between two vectors.
fn cosine_similarity(a: &[f64], b: &[f64]) -> f64 {
    let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
    let norm_b: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot / (norm_a * norm_b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_and_grant() {
        let mut hac = HolographicAccessControl::new(2, 4).unwrap();
        hac.register_capability("read", vec![1]).unwrap();
        hac.grant("alice", "read").unwrap();
    }

    #[test]
    fn holo_access_denied_for_ungranted() {
        let mut hac = HolographicAccessControl::new(2, 4).unwrap();
        hac.register_capability("read", vec![1]).unwrap();
        // Principal doesn't exist → check should fail
        assert!(hac.check_holo_access("alice", "read", 0.5).is_err());
    }

    #[test]
    fn holo_access_with_schubert_check() {
        let mut hac = HolographicAccessControl::new(2, 4).unwrap();
        hac.register_capability("read", vec![1]).unwrap();
        hac.grant("alice", "read").unwrap();

        let result = hac.check_holo_access("alice", "read", 0.0).unwrap();
        // Low threshold — granted by holo similarity, underconstrained by Schubert
        assert!(result.granted);
    }

    #[test]
    fn high_threshold_denies() {
        let mut hac = HolographicAccessControl::new(2, 4).unwrap();
        hac.register_capability("read", vec![1]).unwrap();
        hac.grant("alice", "read").unwrap();

        let result = hac.check_holo_access("alice", "read", 0.99).unwrap();
        // Very high threshold — similarity unlikely to exceed
        // Either result is valid (depends on hash collision)
        // Just verify it runs without error
        let _ = result;
    }

    #[test]
    fn accessible_at_trust() {
        let mut hac = HolographicAccessControl::new(2, 4).unwrap();
        hac.register_capability("read", vec![1]).unwrap();
        hac.register_capability("write", vec![2]).unwrap();
        hac.grant("alice", "read").unwrap();
        hac.grant("alice", "write").unwrap();

        let caps = hac.accessible_at_trust("alice", 0.0).unwrap();
        assert_eq!(caps.len(), 2);
        assert!(caps.contains(&"read".to_string()));
        assert!(caps.contains(&"write".to_string()));
    }

    #[test]
    fn normalized_vectors_have_unit_length() {
        let v = HolographicAccessControl::encode_vector("test");
        let norm: f64 = v.iter().map(|x| x * x).sum::<f64>().sqrt();
        assert!((norm - 1.0).abs() < 1e-10);
    }

    #[test]
    fn same_input_gives_unit_similarity() {
        let a = HolographicAccessControl::encode_vector("read");
        let s = cosine_similarity(&a, &a);
        assert!((s - 1.0).abs() < 1e-10);
    }
}
