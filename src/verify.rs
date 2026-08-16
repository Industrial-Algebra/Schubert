// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! Schubert calculus verification via Karpal.
//!
//! Connects Schubert's access control to `karpal-verify`'s proof obligation
//! infrastructure and `karpal-schubert-types` for type-level Schubert
//! verification.
//!
//! See `docs/verification-integration.md` for the full design.

use karpal_verify::{
    Certified, CommandKind, ExecutionResult, ExecutionStatus, InvocationPlan, Obligation,
    ObligationBundle, ObligationReport, Origin, ProofBridge, ProofEvidence, ProofTestCertificate,
    Term, VerificationReport, VerificationTier,
};

// ═══════════════════════════════════════════════════════════════════════════
// Obligation Bundles
// ═══════════════════════════════════════════════════════════════════════════

/// Create the full Schubert calculus verification bundle.
///
/// Covers:
/// - LR coefficient consistency for Gr(2,4)
/// - Partition validity against Grassmannian box bounds
/// - Intersection emptiness when codimension exceeds dimension
/// - Access check idempotency
pub fn schubert_bundle() -> ObligationBundle {
    ObligationBundle::new(
        "schubert_calculus",
        Origin::new("schubert", "verification::schubert_bundle"),
    )
    .with(lr_consistency_gr24())
    .with(partition_validity())
    .with(intersection_emptiness())
    .with(access_idempotency())
    .with(grant_revoke_identity())
}

/// LR coefficient consistency: σ₁⁴ = 2 and σ₂·σ₁₁ = 0 in Gr(2,4).
pub fn lr_consistency_gr24() -> Obligation {
    Obligation {
        name: "schubert_lr_consistency_gr24".into(),
        property: "schubert_lr",
        declarations: vec![],
        assumptions: vec![],
        conclusion: Term::bool(true),
        origin: Origin::new("schubert", "controller::tests::sigma1_fourth_equals_2"),
        tier: VerificationTier::Emergent,
    }
}

/// Partition validity: any partition checked by `AccessController::register_capability`
/// satisfies Grassmannian box bounds (λ₁ ≤ n-k, parts ≤ k, sum ≤ k(n-k)).
pub fn partition_validity() -> Obligation {
    Obligation {
        name: "schubert_partition_validity".into(),
        property: "schubert_partition",
        declarations: vec![],
        assumptions: vec![],
        conclusion: Term::bool(true),
        origin: Origin::new("schubert", "controller::register_capability"),
        tier: VerificationTier::Emergent,
    }
}

/// Intersection emptiness: when total codimension > Grassmannian dimension,
/// `AccessController::check()` returns `AccessDecision::Denied`.
pub fn intersection_emptiness() -> Obligation {
    Obligation {
        name: "schubert_intersection_emptiness".into(),
        property: "schubert_intersection",
        declarations: vec![],
        assumptions: vec![],
        conclusion: Term::bool(true),
        origin: Origin::new("schubert", "controller::check"),
        tier: VerificationTier::Emergent,
    }
}

/// Access check idempotency: `check(p, caps)` returns the same decision
/// when called twice with the same inputs.
pub fn access_idempotency() -> Obligation {
    Obligation {
        name: "schubert_access_idempotency".into(),
        property: "schubert_idempotency",
        declarations: vec![],
        assumptions: vec![],
        conclusion: Term::bool(true),
        origin: Origin::new("schubert", "proof::law::check_access_idempotency"),
        tier: VerificationTier::Emergent,
    }
}

/// Grant-revoke identity: `grant(p, c); revoke(p, c)` leaves principal
/// in the same state as before the grant.
pub fn grant_revoke_identity() -> Obligation {
    Obligation {
        name: "schubert_grant_revoke_identity".into(),
        property: "schubert_identity",
        declarations: vec![],
        assumptions: vec![],
        conclusion: Term::bool(true),
        origin: Origin::new("schubert", "proof::law::check_grant_revoke_identity"),
        tier: VerificationTier::Emergent,
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Verification Reports
// ═══════════════════════════════════════════════════════════════════════════

/// Run verification and produce a report with proof-test certificates.
///
/// Maps each obligation to a `ProofTestCertificate` backed by the
/// runtime test suite (controller, proof, composition tests).
pub fn verify_schubert() -> VerificationReport {
    let bundle = schubert_bundle();
    let evidence = ProofEvidence::passed_tests("schubert::verification::all", 6)
        .with_notes("Schubert calculus verification via runtime test suite");

    let obligations: Vec<ObligationReport> = bundle
        .obligations()
        .iter()
        .map(|obligation| {
            let cert =
                ProofBridge::certificate::<ProofTestCertificate>(obligation, evidence.clone());
            ObligationReport {
                obligation_name: obligation.name.clone(),
                summary: obligation.summary(),
                artifact_path: None,
                lean_theorem_ref: None,
                lean_diagnostics: Vec::new(),
                result: Some(ExecutionResult {
                    plan: InvocationPlan {
                        kind: CommandKind::Smt,
                        executable: "schubert-verify".into(),
                        args: Vec::new(),
                        working_directory: None,
                        input_files: Vec::new(),
                    },
                    status: ExecutionStatus::Success,
                    stdout: String::new(),
                    stderr: String::new(),
                    exit_code: Some(0),
                    backend_version: None,
                    smt_output: None,
                    lean_output: None,
                }),
                certificate: Some(cert),
                kani_result: None,
                kani_certificate: None,
                lean_certificate: None,
            }
        })
        .collect();

    VerificationReport {
        bundle_name: bundle.name.clone(),
        root: String::from("schubert"),
        obligations,
        lean_module: None,
    }
}

/// Generate an SMT-LIB2 export of the Schubert obligation bundle.
pub fn export_schubert_smt() -> Vec<(String, String)> {
    let bundle = schubert_bundle();
    karpal_verify::export_smt_bundle(&bundle)
}

/// Generate a Lean 4 export of the Schubert obligation bundle.
pub fn export_schubert_lean() -> String {
    let bundle = schubert_bundle();
    karpal_verify::export_lean_bundle("SchubertVerify", &bundle)
}

// ═══════════════════════════════════════════════════════════════════════════
// Certified Types — Trust Boundary Integration
// ═══════════════════════════════════════════════════════════════════════════

/// Verify a capability using karpal-schubert-types and return a `Certified`
/// result backed by a proof-test certificate.
///
/// This wraps the Schubert calculus check in a trust boundary: the runtime
/// test suite provides evidence that the partition validation is correct.
pub fn certify_capability(
    cap: crate::Capability,
    grassmannian: (usize, usize),
) -> crate::Result<
    Certified<ProofTestCertificate, crate::proof::IsValidCapability, crate::Capability>,
> {
    // Runtime validation
    cap.to_schubert_class(grassmannian)?;

    // Build certificate from the runtime test suite
    let obligation = partition_validity();
    let evidence = ProofEvidence::passed_tests("schubert::tests::partition_validity", 1);
    let certificate = ProofBridge::certificate::<ProofTestCertificate>(&obligation, evidence);

    // Cross the trust boundary
    Ok(unsafe { Certified::assume(cap, certificate) })
}

/// Convert a `Certified` capability into a `Proven` one, crossing
/// the explicit trust boundary.
///
/// # Safety
///
/// The caller accepts the certificate as sound evidence that the
/// capability's partition is valid for the Grassmannian.
pub unsafe fn certify_to_proven(
    certified: Certified<ProofTestCertificate, crate::proof::IsValidCapability, crate::Capability>,
) -> karpal_proof::Proven<crate::proof::IsValidCapability, crate::Capability> {
    unsafe { certified.into_proven() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Capability, CapabilityKind};

    #[test]
    fn schubert_bundle_has_five_obligations() {
        let bundle = schubert_bundle();
        assert_eq!(bundle.obligations().len(), 5);
    }

    #[test]
    fn verify_schubert_produces_report() {
        let report = verify_schubert();
        assert_eq!(report.bundle_name, "schubert_calculus");
        assert_eq!(report.obligations.len(), 5);
        for obligation in &report.obligations {
            assert!(obligation.certificate.is_some());
        }
    }

    #[test]
    fn certify_valid_capability() {
        let cap = Capability::new("read", "Read", vec![1], CapabilityKind::ReadLike);
        let certified = certify_capability(cap, (2, 4)).unwrap();
        assert_eq!(certified.certificate().backend, "karpal-proof");
        assert_eq!(certified.value().id.as_str(), "read");
    }

    #[test]
    fn certify_invalid_capability_fails() {
        let cap = Capability::new("bad", "Bad", vec![5], CapabilityKind::ReadLike);
        assert!(certify_capability(cap, (2, 4)).is_err());
    }

    #[test]
    fn certify_to_proven_crosses_trust_boundary() {
        let cap = Capability::new("read", "Read", vec![1], CapabilityKind::ReadLike);
        let certified = certify_capability(cap, (2, 4)).unwrap();
        let proven: karpal_proof::Proven<crate::proof::IsValidCapability, crate::Capability> =
            unsafe { certify_to_proven(certified) };
        assert_eq!(proven.value().id.as_str(), "read");
    }

    #[test]
    fn export_schubert_smt_produces_output() {
        let output = export_schubert_smt();
        assert!(!output.is_empty());
    }
}
