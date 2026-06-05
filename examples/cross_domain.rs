// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: AGPL-3.0-only

//! Multi-Grassmannian cross-domain access control.
//!
//! Run with: `cargo run --example cross_domain`

use schubert::{Capability, CapabilityKind, MultiController};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Multi-Grassmannian Cross-Domain Access ===\n");

    let mut mc = MultiController::new();

    mc.add_domain_named(2, 4, "rbac")?;
    mc.add_domain_named(3, 6, "tenant")?;
    mc.add_domain_named(1, 3, "simple")?;

    println!("Domains registered:");
    for label in mc.domain_labels() {
        let (k, n) = mc.domain(label).unwrap().grassmannian();
        println!("  {label}: Gr({k},{n}) — dimension {}", k * (n - k));
    }
    println!();

    for (domain, caps) in [
        (
            "rbac",
            vec![
                ("read", vec![1], CapabilityKind::ReadLike),
                ("write", vec![2], CapabilityKind::WriteLike),
                ("admin", vec![2, 2], CapabilityKind::AdminLike),
            ],
        ),
        (
            "tenant",
            vec![
                ("read", vec![1], CapabilityKind::ReadLike),
                ("write", vec![2], CapabilityKind::WriteLike),
                ("manage", vec![2, 1], CapabilityKind::AdminLike),
            ],
        ),
        ("simple", vec![("read", vec![1], CapabilityKind::ReadLike)]),
    ] {
        for (id, partition, kind) in caps {
            mc.register_in_domain(Capability::new(id, id, partition, kind), domain)?;
        }
    }

    let alice = mc.create_principal("alice", "rbac")?;
    mc.grant_in_domain(&alice, "read", "rbac")?;
    mc.grant_in_domain(&alice, "write", "rbac")?;

    let bob = mc.create_principal("bob", "tenant")?;
    mc.grant_in_domain(&bob, "read", "tenant")?;

    println!("=== Same-Domain Checks ===\n");
    let d = mc.check_in_domain(&alice, &["read"], "rbac")?;
    println!("alice → read in rbac:          {d:?}");
    let d = mc.check_in_domain(&alice, &["read", "write"], "rbac")?;
    println!("alice → read+write in rbac:    {d:?}");
    let d = mc.check_in_domain(&alice, &["admin"], "rbac")?;
    println!("alice → admin in rbac:         {d:?}");

    println!("\n=== Cross-Domain Checks ===\n");
    let d = mc.check_cross_domain(&alice, &["read"], "rbac", "tenant")?;
    println!("alice: rbac→tenant via read:   {d:?}");
    let d = mc.check_cross_domain(&alice, &["read"], "rbac", "simple")?;
    println!("alice: rbac→simple via read:   {d:?}");
    let d = mc.check_cross_domain(&alice, &["write"], "rbac", "tenant")?;
    println!("alice: rbac→tenant via write:  {d:?}");
    let d = mc.check_cross_domain(&bob, &["read"], "tenant", "rbac")?;
    println!("bob:   tenant→rbac via read:   {d:?}");
    let d = mc.check_cross_domain(&alice, &["admin"], "rbac", "tenant")?;
    println!("alice: rbac→tenant via admin:  {d:?}");

    println!("\n=== Translatable Capabilities ===\n");
    let translatable = mc.translatable_capabilities("rbac", "tenant")?;
    println!("RBAC → Tenant translatable: {translatable:?}");
    let domains = mc.domains_for_partition(&[1]);
    println!("Domains accepting sigma1: {domains:?}");
    Ok(())
}
