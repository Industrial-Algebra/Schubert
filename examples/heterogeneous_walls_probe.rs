// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! Heterogeneous walls probe — Thatch probe S4 (trust×time product).
//!
//! The first instrument that crosses a **time wall** (grant expiry, in
//! cryptographic space) with a **trust wall** (partition stability, in
//! controller space) in one composition harness. It extends the Roadmap #50
//! baseline ("all compositions additive — walls are per-capability") from the
//! pure-trust axis to the trust×time product.
//!
//! The seam under test: the expiring capability `temp_sign` shares partition
//! λ=[2] with the permanent capability `audit`. Under per-capability walls the
//! λ=[2] wall position in the composed diagram must survive `temp_sign`'s
//! death (`audit` holds it). If the breakpoint set changes beyond the
//! predictable capability-removal, that is heterogeneous emergence — a
//! *finding*, not a bug.
//!
//! Semantic bridge (ADR-0001): expired grant ⇒ capability dead ⇒ principal
//! rebuilt without it. `verify_at` instruments the time wall; a second
//! controller rebuild instruments the simulated revocation. No engine or API
//! changes.
//!
//! Run:
//! ```text
//! cargo run --example heterogeneous_walls_probe --features crypto
//! ```

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

fn main() -> schubert::Result<()> {
    // Time wall: an expiring capability grant in cryptographic space.
    let issuer = CapabilityIssuer::generate();
    let verifier = GrantVerifier::new(issuer.public_key())?;

    let acl_pre = build_controller(true)?;
    let pa = PrincipalId::new("wall-a")?;
    let pb = PrincipalId::new("wall-b")?;

    let grant = issuer.issue_grant_with_expiry(
        pa.clone(),
        &[(CapabilityId::new("temp_sign")?, vec![2])],
        1000,
    )?;

    let valid = verifier.verify_at(&grant, 999);
    println!("Time wall: verify_at(temp_sign grant, now=999) -> {valid:?}");
    assert!(valid.is_ok(), "grant must be valid before expiry");

    let expired = verifier.verify_at(&grant, 1000);
    println!("Time wall: verify_at(temp_sign grant, now=1000) -> {expired:?}");
    assert!(
        matches!(expired, Err(SchubertError::GrantExpired { .. })),
        "verify_at is inclusive: now >= expires_at kills the grant"
    );

    // Trust wall: composed stability, pre- and post-cliff.
    let report_pre = analyze_composed_stability(&acl_pre, &pa, "read", &pb, "read")?;

    // Post-cliff: principal rebuilt without temp_sign (ADR-0001 semantic
    // bridge). No grant is needed — the capability is simply gone.
    let acl_post = build_controller(false)?;
    let report_post = analyze_composed_stability(&acl_post, &pa, "read", &pb, "read")?;

    println!("\nHeterogeneous walls probe — Thatch S4 (Gr(2,4))");
    println!("{}", "=".repeat(64));

    let slices: [(&str, &ComposedStabilityReport, &str); 2] = [
        ("pre-cliff ", &report_pre, "temp_sign alive"),
        ("post-cliff", &report_post, "temp_sign dead"),
    ];
    for (label, report, note) in slices {
        println!(
            "\n[{}] {}\n  composed caps: {} | additive: {} | breakpoints: {:?}",
            label,
            note,
            report.phase_diagram_composed.total_capabilities,
            report.is_additive,
            breakpoint_values(report),
        );
    }

    let pre_bps = breakpoint_values(&report_pre);
    let post_bps = breakpoint_values(&report_post);

    println!("\n{}", "-".repeat(64));
    if pre_bps == post_bps {
        println!("WALLS SUPERPOSE — the λ=[2] wall survives temp_sign's death via");
        println!("audit; the composed breakpoint set is unchanged across the time cliff.");
    } else {
        println!("HETEROGENEOUS EMERGENCE DETECTED — report both breakpoint sets.");
        println!("  pre-cliff  breakpoints: {:?}", pre_bps);
        println!("  post-cliff breakpoints: {:?}", post_bps);
    }

    if pre_bps == post_bps {
        println!("\nRESULT: trust×time product is additive — under the current engine");
        println!("baseline, the composed λ=[2] wall is unchanged by temp_sign's death;");
        println!("the wall position is held by audit, not by the expiring grant. No");
        println!("heterogeneous emergence in the trust×time product at this resolution.");
    } else {
        println!("\nRESULT: heterogeneous emergence in the trust×time product — the");
        println!("composed breakpoint set is not determined by per-capability walls.");
        println!("The deviation above is the finding; report it as such.");
    }

    Ok(())
}
