// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! Operadic composition of principals.
//!
//! Two principals can be composed when one's output capability matches
//! the other's input capability. The composition consumes the interface
//! capabilities and merges the remaining access spaces, producing a
//! multiplicity that counts how many configurations survive the composition.
//!
//! # Mathematical Background
//!
//! Operadic composition in the Schubert calculus context models service
//! chaining: principal A produces output that principal B consumes.
//! The composed access policy is the intersection after gluing along
//! the shared interface. The multiplicity is the pushforward degree —
//! how many configurations of the composition correspond to each
//! configuration of the composed principal.
//!
//! # Example
//!
//! ```ignore
//! use schubert::{AccessController, Capability, CapabilityKind, compose, are_composable};
//!
//! let mut acl = AccessController::new(2, 4)?;
//! acl.register_capability(Capability::new(
//!     "output:data", "Output", vec![1], CapabilityKind::ReadLike,
//! ))?;
//! acl.register_capability(Capability::new(
//!     "input:data", "Input", vec![1], CapabilityKind::ReadLike,
//! ))?;
//!
//! let producer = acl.create_principal("producer")?;
//! let consumer = acl.create_principal("consumer")?;
//! acl.grant(&producer, "output:data")?;
//! acl.grant(&consumer, "input:data")?;
//!
//! let result = compose(&acl, &producer, "output:data", &consumer, "input:data")?;
//! // result.multiplicity — how many configurations survive
//! // result.retained_capabilities — non-interface caps from both
//! # Ok::<(), schubert::SchubertError>(())
//! ```

use crate::controller::AccessController;
use crate::error::{Result, SchubertError};
use crate::principal::PrincipalId;
use amari_enumerative::{composition_multiplicity, ComposableNamespace};

/// Result of composing two principals via a shared interface.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CompositionResult {
    /// Number of configurations for the composed access space.
    ///
    /// The pushforward degree of the operadic composition. Values > 1
    /// indicate multiple valid configurations survive the gluing.
    pub multiplicity: u64,
    /// Non-interface capability IDs retained by the composition.
    ///
    /// Capabilities that were not consumed as the interface are
    /// preserved in the composed principal.
    pub retained_capabilities: Vec<String>,
}

/// Compose two principals via a shared interface capability.
///
/// Principal A's output capability is consumed as principal B's input.
/// The composed result retains non-interface capabilities from both.
/// The multiplicity reflects the pushforward degree — the number of
/// configurations of A that, when composed with B, produce a valid result.
///
/// # Errors
///
/// Returns `PrincipalNotFound` if either principal does not exist.
/// Returns `CapabilityNotHeld` if either principal lacks its interface capability.
/// Returns `CompositionFailed` if the composition cannot be computed.
pub fn compose(
    acl: &AccessController,
    principal_a: &PrincipalId,
    output_cap: &str,
    principal_b: &PrincipalId,
    input_cap: &str,
) -> Result<CompositionResult> {
    let a = acl
        .principal(principal_a)
        .ok_or_else(|| SchubertError::PrincipalNotFound(principal_a.to_string()))?;
    let b = acl
        .principal(principal_b)
        .ok_or_else(|| SchubertError::PrincipalNotFound(principal_b.to_string()))?;

    if !a.holds(output_cap) {
        return Err(SchubertError::CapabilityNotHeld {
            principal: principal_a.to_string(),
            capability: output_cap.to_string(),
        });
    }
    if !b.holds(input_cap) {
        return Err(SchubertError::CapabilityNotHeld {
            principal: principal_b.to_string(),
            capability: input_cap.to_string(),
        });
    }

    let output_idx = a
        .namespace
        .capabilities
        .iter()
        .position(|c| c.id.as_str() == output_cap)
        .ok_or_else(|| SchubertError::CapabilityNotHeld {
            principal: principal_a.to_string(),
            capability: output_cap.to_string(),
        })?;
    let input_idx = b
        .namespace
        .capabilities
        .iter()
        .position(|c| c.id.as_str() == input_cap)
        .ok_or_else(|| SchubertError::CapabilityNotHeld {
            principal: principal_b.to_string(),
            capability: input_cap.to_string(),
        })?;

    let mut comp_a = ComposableNamespace::new(a.namespace.clone());
    let mut comp_b = ComposableNamespace::new(b.namespace.clone());
    let amari_out = amari_enumerative::CapabilityId::new(output_cap);
    let amari_in = amari_enumerative::CapabilityId::new(input_cap);
    comp_a
        .mark_output(&amari_out)
        .map_err(SchubertError::CompositionFailed)?;
    comp_b
        .mark_input(&amari_in)
        .map_err(SchubertError::CompositionFailed)?;

    let multiplicity = composition_multiplicity(&comp_a, output_idx, &comp_b, input_idx);

    let mut retained = Vec::new();
    for (i, cap) in a.namespace.capabilities.iter().enumerate() {
        if i != output_idx {
            retained.push(cap.id.to_string());
        }
    }
    for (i, cap) in b.namespace.capabilities.iter().enumerate() {
        if i != input_idx {
            retained.push(cap.id.to_string());
        }
    }

    Ok(CompositionResult {
        multiplicity,
        retained_capabilities: retained,
    })
}

/// Check whether two principals are composable via their interfaces.
///
/// Returns `true` if principal A holds `output_cap` and principal B
/// holds `input_cap`. Does not compute the composition — only tests
/// the structural precondition.
pub fn are_composable(
    acl: &AccessController,
    principal_a: &PrincipalId,
    output_cap: &str,
    principal_b: &PrincipalId,
    input_cap: &str,
) -> bool {
    let a = acl.principal(principal_a);
    let b = acl.principal(principal_b);
    a.is_some_and(|a| a.holds(output_cap)) && b.is_some_and(|b| b.holds(input_cap))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability::CapabilityKind;
    use crate::Capability;

    fn make_acl() -> AccessController {
        let mut acl = AccessController::new(2, 4).unwrap();
        for (id, partition, kind) in [
            ("out", vec![1], CapabilityKind::ReadLike),
            ("in", vec![1], CapabilityKind::ReadLike),
            ("keep_a", vec![1], CapabilityKind::ReadLike),
            ("keep_b", vec![1], CapabilityKind::ReadLike),
        ] {
            acl.register_capability(Capability::new(id, id, partition, kind))
                .unwrap();
        }
        acl
    }

    #[test]
    fn compose_two_principals() {
        let mut acl = make_acl();
        let a = acl.create_principal("producer").unwrap();
        let b = acl.create_principal("consumer").unwrap();
        acl.grant(&a, "out").unwrap();
        acl.grant(&a, "keep_a").unwrap();
        acl.grant(&b, "in").unwrap();
        acl.grant(&b, "keep_b").unwrap();

        let result = compose(&acl, &a, "out", &b, "in").unwrap();
        assert!(result.multiplicity > 0);
        assert!(result.retained_capabilities.contains(&"keep_a".to_string()));
        assert!(result.retained_capabilities.contains(&"keep_b".to_string()));
    }

    #[test]
    fn compose_without_interface_fails() {
        let mut acl = make_acl();
        let a = acl.create_principal("a").unwrap();
        let b = acl.create_principal("b").unwrap();
        assert!(compose(&acl, &a, "out", &b, "in").is_err());
    }

    #[test]
    fn are_composable_detects_compatibility() {
        let mut acl = make_acl();
        let a = acl.create_principal("a").unwrap();
        let b = acl.create_principal("b").unwrap();
        acl.grant(&a, "out").unwrap();
        acl.grant(&b, "in").unwrap();
        assert!(are_composable(&acl, &a, "out", &b, "in"));
        assert!(!are_composable(&acl, &a, "out", &b, "out"));
    }
}
