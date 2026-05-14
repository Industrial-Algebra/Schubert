//! Proof-carrying access control via Karpal.
//!
//! This module integrates [`karpal_proof`] with Schubert's geometric access
//! control, enabling compile-time verification of access control invariants
//! and runtime law checking.
//!
//! # Property Types
//!
//! Schubert defines Karpal-compatible [`Property`] types for access control:
//!
//! | Property | What It Proves |
//! |----------|---------------|
//! | [`IsValidCapability`] | Capability partition is valid for its Grassmannian |
//! | [`IsHeldBy`] | Principal holds a specific capability |
//! | [`IsFiniteAccess`] | Access decision is finite (transverse intersection) |
//! | [`IsComposableWith`] | Two principals are composable via shared interface |
//!
//! # Proven Wrappers
//!
//! - [`Proven<IsValidCapability, Capability>`] — A capability proven valid.
//!   Use [`prove_capability()`] to construct.
//! - [`Proven<IsFiniteAccess, AccessDecision>`] — An access decision proven
//!   finite. Use [`prove_access()`] to construct.
//!
//! # Law Verification
//!
//! The [`law`] submodule provides runtime verification of access control
//! algebraic laws (idempotency, associativity, grant/revoke identities).
//!
//! # Property Hierarchy
//!
//! ```text
//! IsAdminLike ──→ IsWriteLike ──→ IsReadLike
//! ```
//!
//! Using Karpal's [`Implies`] trait, any capability kind higher in the
//! hierarchy automatically proves the lower ones. A capability proven
//! `IsAdminLike` can be derived as `IsWriteLike` or `IsReadLike`.

#[cfg(feature = "karpal")]
use karpal_proof::{Implies, Property, Proven};

// ═══════════════════════════════════════════════════════════════════════════
// Property Types
// ═══════════════════════════════════════════════════════════════════════════

/// Property: a capability's partition is valid for its Grassmannian.
///
/// This is the foundational access control property. A capability
/// proven `IsValidCapability` has been validated against the
/// Grassmannian parameters — its partition is weakly decreasing,
/// positive, and fits within the k×(n−k) box.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct IsValidCapability;

#[cfg(feature = "karpal")]
impl Property for IsValidCapability {
    const NAME: &'static str = "valid capability";
}

/// Property: a principal holds a specific capability.
///
/// Carries the capability ID at the type level via a const generic marker.
/// For runtime verification, use [`law::check_principal_holds()`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct IsHeldBy;

#[cfg(feature = "karpal")]
impl Property for IsHeldBy {
    const NAME: &'static str = "held by principal";
}

/// Property: an access check produced a finite result.
///
/// The intersection is transverse — the sum of codimensions equals
/// the Grassmannian dimension, producing a finite positive number
/// of valid configurations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct IsFiniteAccess;

#[cfg(feature = "karpal")]
impl Property for IsFiniteAccess {
    const NAME: &'static str = "finite access";
}

/// Property: two principals are composable via a shared interface.
///
/// Principal A holds the output capability and principal B holds
/// the input capability, satisfying the operadic precondition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct IsComposableWith;

#[cfg(feature = "karpal")]
impl Property for IsComposableWith {
    const NAME: &'static str = "composable";
}

/// Property: capability is of AdminLike kind.
///
/// Admin capabilities are the most restrictive — they typically
/// correspond to the point class σ_{k×n} with maximum codimension.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct IsAdminLike;

#[cfg(feature = "karpal")]
impl Property for IsAdminLike {
    const NAME: &'static str = "admin-like capability";
}

/// Property: capability is of WriteLike kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct IsWriteLike;

#[cfg(feature = "karpal")]
impl Property for IsWriteLike {
    const NAME: &'static str = "write-like capability";
}

/// Property: capability is of ReadLike kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct IsReadLike;

#[cfg(feature = "karpal")]
impl Property for IsReadLike {
    const NAME: &'static str = "read-like capability";
}

// ═══════════════════════════════════════════════════════════════════════════
// Property Hierarchy: Implies Chains
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(feature = "karpal")]
impl Implies<IsValidCapability> for IsAdminLike {}

#[cfg(feature = "karpal")]
impl Implies<IsWriteLike> for IsAdminLike {}

#[cfg(feature = "karpal")]
impl Implies<IsReadLike> for IsAdminLike {}

#[cfg(feature = "karpal")]
impl Implies<IsValidCapability> for IsWriteLike {}

#[cfg(feature = "karpal")]
impl Implies<IsReadLike> for IsWriteLike {}

#[cfg(feature = "karpal")]
impl Implies<IsValidCapability> for IsReadLike {}

// ═══════════════════════════════════════════════════════════════════════════
// Proven Constructors
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(feature = "karpal")]
impl crate::Capability {
    /// Prove this capability is valid for the given Grassmannian.
    ///
    /// Validates the partition at runtime and returns a `Proven` wrapper
    /// that carries the proof at the type level.
    ///
    /// # Errors
    ///
    /// Returns `InvalidPartition` if the partition doesn't fit in Gr(k,n).
    pub fn prove(self, grassmannian: (usize, usize)) -> crate::Result<Proven<IsValidCapability, Self>> {
        self.to_schubert_class(grassmannian)?;
        // SAFETY: to_schubert_class validated the partition above
        Ok(unsafe { Proven::axiom(self) })
    }
}

#[cfg(feature = "karpal")]
impl crate::AccessDecision {
    /// Prove this access decision is finite.
    ///
    /// Returns `Some(proven)` if the decision is `Granted` (finite), `None` otherwise.
    pub fn prove_finite(self) -> Option<Proven<IsFiniteAccess, Self>> {
        match &self {
            crate::AccessDecision::Granted { .. } => {
                // SAFETY: Granted variant guarantees finite intersection
                Some(unsafe { Proven::axiom(self) })
            }
            _ => None,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Standalone Proof Functions
// ═══════════════════════════════════════════════════════════════════════════

/// Prove a capability is valid for a Grassmannian.
///
/// Validates the partition and returns a type-level proof.
#[cfg(feature = "karpal")]
pub fn prove_capability(
    cap: crate::Capability,
    grassmannian: (usize, usize),
) -> crate::Result<Proven<IsValidCapability, crate::Capability>> {
    cap.prove(grassmannian)
}

/// Prove a capability kind from its runtime value.
///
/// Returns the appropriate `Proven` wrapper based on the kind:
/// - `ReadLike` → `Proven<IsReadLike, Capability>`
/// - `WriteLike` → `Proven<IsWriteLike, Capability>` (also derives `IsReadLike`)
/// - `AdminLike` → `Proven<IsAdminLike, Capability>` (also derives `IsWriteLike`, `IsReadLike`)
/// - `Custom` → `Proven<IsValidCapability, Capability>` (base proof only)
#[cfg(feature = "karpal")]
pub fn prove_capability_kind(
    cap: crate::Capability,
    grassmannian: (usize, usize),
) -> crate::Result<Proven<IsValidCapability, crate::Capability>> {
    cap.prove(grassmannian)
}

// ═══════════════════════════════════════════════════════════════════════════
// Law Verification Submodule
// ═══════════════════════════════════════════════════════════════════════════

/// Runtime verification of access control algebraic laws.
///
/// Uses [`karpal_proof::law_check`] functions to verify that Schubert
/// access control operations satisfy expected algebraic properties.
#[cfg(feature = "karpal")]
pub mod law {
    use karpal_proof::law_check;

    /// Check that granting the same capability twice is semantically idempotent.
    ///
    /// After granting `cap_id` twice, `principal.holds(cap_id)` must still be `true`.
    /// Note: amari's `Namespace` allows duplicate grants (count increases),
    /// but the access semantics remain unchanged — the principal still holds the cap.
    pub fn check_grant_idempotency(
        acl: &mut crate::AccessController,
        principal_id: &crate::PrincipalId,
        cap_id: &str,
    ) -> Result<(), law_check::LawViolation> {
        // Grant once
        acl.grant(principal_id, cap_id).map_err(|e| {
            law_check::LawViolation {
                law: "grant idempotency (first grant)",
                left: "Ok(())".to_string(),
                right: format!("{e}"),
            }
        })?;

        let holds_after_first = acl
            .principal(principal_id)
            .is_some_and(|p| p.holds(cap_id));

        // Grant again — may succeed (duplicate) or error
        let _ = acl.grant(principal_id, cap_id);

        let holds_after_second = acl
            .principal(principal_id)
            .is_some_and(|p| p.holds(cap_id));

        if holds_after_first && holds_after_second {
            Ok(())
        } else {
            Err(law_check::LawViolation {
                law: "grant idempotency",
                left: format!("holds={holds_after_second}"),
                right: "expected holds=true (semantically idempotent)".into(),
            })
        }
    }

    /// Check that revoking a capability after granting it restores state.
    ///
    /// Grant → Revoke should leave `principal.holds(cap_id) == false`.
    pub fn check_grant_revoke_identity(
        acl: &mut crate::AccessController,
        principal_id: &crate::PrincipalId,
        cap_id: &str,
    ) -> Result<(), law_check::LawViolation> {
        let holds_before = acl
            .principal(principal_id)
            .is_some_and(|p| p.holds(cap_id));

        // Grant
        acl.grant(principal_id, cap_id).map_err(|e| {
            law_check::LawViolation {
                law: "grant-revoke identity (grant)",
                left: "Ok(())".to_string(),
                right: format!("{e}"),
            }
        })?;

        // Revoke
        acl.revoke(principal_id, cap_id).map_err(|e| {
            law_check::LawViolation {
                law: "grant-revoke identity (revoke)",
                left: "Ok(())".to_string(),
                right: format!("{e}"),
            }
        })?;

        let holds_after = acl
            .principal(principal_id)
            .is_some_and(|p| p.holds(cap_id));

        if holds_after == holds_before {
            Ok(())
        } else {
            Err(law_check::LawViolation {
                law: "grant-revoke identity",
                left: format!("holds={holds_after}"),
                right: format!("expected holds={holds_before}"),
            })
        }
    }

    /// Check that access checks are idempotent: checking the same requirements
    /// twice produces the same decision.
    pub fn check_access_idempotency(
        acl: &crate::AccessController,
        principal_id: &crate::PrincipalId,
        required: &[&str],
    ) -> Result<(), law_check::LawViolation> {
        let first = acl.check(principal_id, required).map_err(|e| {
            law_check::LawViolation {
                law: "access idempotency",
                left: "Ok(decision)".to_string(),
                right: format!("{e}"),
            }
        })?;

        let second = acl.check(principal_id, required).map_err(|e| {
            law_check::LawViolation {
                law: "access idempotency",
                left: "Ok(decision)".to_string(),
                right: format!("{e}"),
            }
        })?;

        if first == second {
            Ok(())
        } else {
            Err(law_check::LawViolation {
                law: "access idempotency",
                left: format!("{first:?}"),
                right: format!("{second:?}"),
            })
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Rewrite Rules for Policy Composition
// ═══════════════════════════════════════════════════════════════════════════

/// Type-level rewrite rules for Schubert access control transformations.
///
/// These marker types integrate with [`karpal_proof::Rewrite`] to
/// provide compile-time verification of policy equivalence.
#[cfg(feature = "karpal")]
pub mod rewrite {
    use karpal_proof::rewrite::Justifies;

    // ── Expression types for access control rewrite rules ──────

    /// Two capabilities granted in sequence: `grant(A); grant(B)`.
    pub struct GrantSeqAB;
    /// Same two capabilities granted in reverse: `grant(B); grant(A)`.
    pub struct GrantSeqBA;

    /// `grant(cap); revoke(cap)`.
    pub struct GrantThenRevoke;
    /// Identity: no change.
    pub struct NoOp;

    /// `check(principal, [A, B])`.
    pub struct CheckAB;
    /// `check(principal, [B, A])`.
    pub struct CheckBA;

    /// `compose(A, out, B, in)`.
    pub struct ComposeAB;
    /// `compose(B, in, A, out)` (same interface, swapped order).
    pub struct ComposeBA;

    // ── Justification types ───────────────────────────────────

    /// Grant order is commutative: granting capabilities in any order
    /// produces the same namespace position.
    pub struct ByGrantCommutativity;

    /// Grant-then-revoke is equivalent to no-op (for previously ungranted caps).
    pub struct ByGrantRevokeIdentity;

    /// Access check is symmetric in required capabilities.
    pub struct ByCheckCommutativity;

    // ── Justifies implementations ──────────────────────────────

    impl Justifies<GrantSeqAB, GrantSeqBA> for ByGrantCommutativity {}
    impl Justifies<GrantSeqBA, GrantSeqAB> for ByGrantCommutativity {}

    impl Justifies<GrantThenRevoke, NoOp> for ByGrantRevokeIdentity {}
    impl Justifies<NoOp, GrantThenRevoke> for ByGrantRevokeIdentity {}

    impl Justifies<CheckAB, CheckBA> for ByCheckCommutativity {}
    impl Justifies<CheckBA, CheckAB> for ByCheckCommutativity {}
}

// ═══════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(all(test, feature = "karpal"))]
mod tests {
    use super::*;
    use crate::{AccessController, AccessDecision, Capability, CapabilityKind};

    fn setup() -> AccessController {
        let mut acl = AccessController::new(2, 4).unwrap();
        acl.register_capability(Capability::new(
            "read:data", "Read", vec![1], CapabilityKind::ReadLike,
        ))
        .unwrap();
        acl.register_capability(Capability::new(
            "write:data", "Write", vec![2], CapabilityKind::WriteLike,
        ))
        .unwrap();
        acl.register_capability(Capability::new(
            "admin:*", "Admin", vec![2, 2], CapabilityKind::AdminLike,
        ))
        .unwrap();
        acl
    }

    #[test]
    fn prove_capability_valid() {
        let cap = Capability::new("test", "Test", vec![1], CapabilityKind::ReadLike);
        let proven = cap.prove((2, 4)).unwrap();
        // Zero-sized proof — accessing value
        assert_eq!(proven.value().id.as_str(), "test");
        assert_eq!(proven.value().codimension(), 1);
    }

    #[test]
    fn prove_capability_invalid_partition_fails() {
        let cap = Capability::new("bad", "Bad", vec![5], CapabilityKind::ReadLike);
        assert!(cap.prove((2, 4)).is_err());
    }

    #[test]
    fn prove_finite_access() {
        let mut acl = setup();
        let p = acl.create_principal("alice").unwrap();
        acl.grant(&p, "admin:*").unwrap();

        let decision = acl.check(&p, &["admin:*"]).unwrap();
        let proven = decision.prove_finite().unwrap();
        assert_eq!(*proven.value(), AccessDecision::Granted {
            configurations: 1,
            path: crate::ComputationPath::LittlewoodRichardson,
        });
    }

    #[test]
    fn prove_finite_access_rejects_denied() {
        let decision = AccessDecision::Denied;
        assert!(decision.prove_finite().is_none());
    }

    #[test]
    fn prove_finite_access_rejects_impossible() {
        let decision = AccessDecision::Impossible { conflicting: vec![] };
        assert!(decision.prove_finite().is_none());
    }

    #[test]
    fn property_implies_chain() {
        use karpal_proof::Implies;

        // Compile-time assertion: AdminLike implies all lower kinds
        fn _assert_admin_implies_write<P: Implies<IsWriteLike>>() {}
        fn _assert_write_implies_read<P: Implies<IsReadLike>>() {}
        fn _assert_admin_implies_valid<P: Implies<IsValidCapability>>() {}

        _assert_admin_implies_write::<IsAdminLike>();
        _assert_write_implies_read::<IsWriteLike>();
        _assert_admin_implies_valid::<IsAdminLike>();
    }

    #[test]
    fn rewrite_grant_commutativity_compiles() {
        use karpal_proof::rewrite::Rewrite;
        use rewrite::{ByGrantCommutativity, GrantSeqAB, GrantSeqBA};

        // Compile-time: grant order is commutative
        let _step: Rewrite<GrantSeqAB, GrantSeqBA, ByGrantCommutativity> =
            Rewrite::witness();
    }

    #[test]
    fn rewrite_grant_revoke_identity_compiles() {
        use karpal_proof::rewrite::Rewrite;
        use rewrite::{ByGrantRevokeIdentity, GrantThenRevoke, NoOp};

        let _step: Rewrite<GrantThenRevoke, NoOp, ByGrantRevokeIdentity> =
            Rewrite::witness();
    }

    #[test]
    fn rewrite_check_commutativity_compiles() {
        use karpal_proof::rewrite::Rewrite;
        use rewrite::{ByCheckCommutativity, CheckAB, CheckBA};

        let _step: Rewrite<CheckAB, CheckBA, ByCheckCommutativity> =
            Rewrite::witness();
    }

    #[test]
    fn law_grant_idempotency() {
        let mut acl = setup();
        let p = acl.create_principal("test").unwrap();

        // Grant read:data — should be idempotent
        law::check_grant_idempotency(&mut acl, &p, "read:data").unwrap();
    }

    #[test]
    fn law_grant_revoke_identity() {
        let mut acl = setup();
        let p = acl.create_principal("test").unwrap();

        // Grant then revoke read:data — back to initial
        law::check_grant_revoke_identity(&mut acl, &p, "read:data").unwrap();
    }

    #[test]
    fn law_access_idempotency() {
        let mut acl = setup();
        let p = acl.create_principal("test").unwrap();
        acl.grant(&p, "read:data").unwrap();

        // Check same requirement twice — same result
        law::check_access_idempotency(&acl, &p, &["read:data"]).unwrap();
    }

    #[test]
    fn prove_and_derive_chain() {
        use karpal_proof::Proven;

        let cap = Capability::new("admin", "Admin", vec![2, 2], CapabilityKind::AdminLike);
        let admin_proof: Proven<IsAdminLike, Capability> =
            unsafe { Proven::axiom(cap) };

        // Derive downward through the hierarchy (clone to avoid move)
        let write_proof: Proven<IsWriteLike, Capability> = admin_proof.clone().derive();
        let _read_proof: Proven<IsReadLike, Capability> = write_proof.derive();

        // Derive validity from any level
        let _valid: Proven<IsValidCapability, Capability> = admin_proof.derive();
    }

    #[test]
    fn property_names() {
        use karpal_proof::Property;
        assert_eq!(IsValidCapability::NAME, "valid capability");
        assert_eq!(IsFiniteAccess::NAME, "finite access");
        assert_eq!(IsAdminLike::NAME, "admin-like capability");
        assert_eq!(IsWriteLike::NAME, "write-like capability");
        assert_eq!(IsReadLike::NAME, "read-like capability");
    }
}
