#![cfg(feature = "crypto")]
// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! Regression tests for the heterogeneous walls probe (Thatch probe S4).
//!
//! Pins the trust×time product baseline: the expiring capability `temp_sign`
//! shares partition λ=[2] with the permanent capability `audit`. When
//! `temp_sign` dies at the time wall, the composed breakpoint set must not
//! move, because `audit` still holds the λ=[2] wall. A deviation would be
//! heterogeneous emergence — a finding, not a bug.

use schubert::crypto::{CapabilityIssuer, GrantVerifier};
use schubert::stability::{analyze_composed_stability, ComposedStabilityReport};
use schubert::{
    AccessController, Capability, CapabilityId, CapabilityKind, PrincipalId, SchubertError,
};

/// Build the fixed S4 scenario controller.
///
/// Registers the four capabilities in contract order, creates `wall-a`
/// (grants `read`, `temp_sign` iff `with_temp_sign`, `audit`) and `wall-b`
/// (grants `read`, `manage`).
fn build_controller(with_temp_sign: bool) -> schubert::Result<AccessController> {
    let mut acl = AccessController::new(2, 4)?;

    let caps: [(String, Vec<usize>, CapabilityKind); 4] = [
        ("read".into(), vec![1], CapabilityKind::ReadLike),
        ("temp_sign".into(), vec![2], CapabilityKind::WriteLike),
        ("audit".into(), vec![2], CapabilityKind::ReadLike),
        ("manage".into(), vec![2, 1], CapabilityKind::WriteLike),
    ];
    for (id, partition, kind) in &caps {
        acl.register_capability(Capability::new(
            CapabilityId::new(id.clone()).expect("valid id"),
            id.clone(),
            partition.clone(),
            *kind,
        ))?;
    }

    let pa = acl.create_principal(PrincipalId::new("wall-a").expect("valid id"))?;
    let pb = acl.create_principal(PrincipalId::new("wall-b").expect("valid id"))?;

    acl.grant(&pa, "read")?;
    if with_temp_sign {
        acl.grant(&pa, "temp_sign")?;
    }
    acl.grant(&pa, "audit")?;

    acl.grant(&pb, "read")?;
    acl.grant(&pb, "manage")?;

    Ok(acl)
}

/// Sorted vector of composed breakpoint trust levels.
fn breakpoint_values(report: &ComposedStabilityReport) -> Vec<f64> {
    let mut values: Vec<f64> = report
        .phase_diagram_composed
        .phase_diagram
        .iter()
        .map(|bp| bp.trust_level.value())
        .collect();
    values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    values
}

/// The fixed pre-cliff composed report plus the issued expiring grant.
fn pre_cliff_fixture() -> (
    CapabilityIssuer,
    GrantVerifier,
    schubert::crypto::GrantToken,
) {
    let issuer = CapabilityIssuer::generate();
    let verifier = GrantVerifier::new(issuer.public_key()).expect("valid public key");
    let grant = issuer
        .issue_grant_with_expiry(
            PrincipalId::new("wall-a").expect("valid id"),
            &[(CapabilityId::new("temp_sign").expect("valid id"), vec![2])],
            1000,
        )
        .expect("expiring grant");
    (issuer, verifier, grant)
}

#[test]
fn grant_valid_before_expiry() {
    let (_issuer, verifier, grant) = pre_cliff_fixture();
    assert!(verifier.verify_at(&grant, 999).is_ok());
}

#[test]
fn grant_expires_inclusively() {
    let (_issuer, verifier, grant) = pre_cliff_fixture();
    assert!(matches!(
        verifier.verify_at(&grant, 1000),
        Err(SchubertError::GrantExpired { .. })
    ));
}

#[test]
fn pre_cliff_composed_capability_count() {
    let acl = build_controller(true).expect("controller");
    let pa = PrincipalId::new("wall-a").expect("valid id");
    let pb = PrincipalId::new("wall-b").expect("valid id");
    let report = analyze_composed_stability(&acl, &pa, "read", &pb, "read").expect("analysis");
    assert_eq!(report.phase_diagram_composed.total_capabilities, 3);
}

#[test]
fn post_cliff_composed_capability_count() {
    let acl = build_controller(false).expect("controller");
    let pa = PrincipalId::new("wall-a").expect("valid id");
    let pb = PrincipalId::new("wall-b").expect("valid id");
    let report = analyze_composed_stability(&acl, &pa, "read", &pb, "read").expect("analysis");
    assert_eq!(report.phase_diagram_composed.total_capabilities, 2);
}

#[test]
fn walls_superpose_across_time_cliff() {
    let acl_pre = build_controller(true).expect("controller");
    let acl_post = build_controller(false).expect("controller");
    let pa = PrincipalId::new("wall-a").expect("valid id");
    let pb = PrincipalId::new("wall-b").expect("valid id");

    let report_pre =
        analyze_composed_stability(&acl_pre, &pa, "read", &pb, "read").expect("pre analysis");
    let report_post =
        analyze_composed_stability(&acl_post, &pa, "read", &pb, "read").expect("post analysis");

    assert_eq!(
        breakpoint_values(&report_pre),
        breakpoint_values(&report_post),
        "the λ=[2] wall must survive temp_sign's death via audit"
    );
}

#[test]
fn ks_additivity_holds_in_both_slices() {
    let acl_pre = build_controller(true).expect("controller");
    let acl_post = build_controller(false).expect("controller");
    let pa = PrincipalId::new("wall-a").expect("valid id");
    let pb = PrincipalId::new("wall-b").expect("valid id");

    let report_pre =
        analyze_composed_stability(&acl_pre, &pa, "read", &pb, "read").expect("pre analysis");
    let report_post =
        analyze_composed_stability(&acl_post, &pa, "read", &pb, "read").expect("post analysis");

    assert!(
        report_pre.is_additive,
        "pre-cliff composition must be additive"
    );
    assert!(
        report_post.is_additive,
        "post-cliff composition must be additive"
    );
}
