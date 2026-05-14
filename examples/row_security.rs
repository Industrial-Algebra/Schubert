// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: AGPL-3.0-only

//! Multi-tenant row-level security example.
//!
//! Models database row-level security with tenant isolation as
//! Schubert conditions. Cross-tenant access is geometrically impossible.

use schubert::{AccessController, AccessDecision, Capability, CapabilityKind};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut acl = AccessController::new(2, 4)?;

    for tenant in &["tenant_a", "tenant_b", "tenant_c"] {
        acl.register_capability(Capability::new(
            format!("read:{tenant}"), format!("Read {tenant}"), vec![1], CapabilityKind::ReadLike,
        ))?;
        acl.register_capability(Capability::new(
            format!("write:{tenant}"), format!("Write {tenant}"), vec![2], CapabilityKind::WriteLike,
        ))?;
    }
    acl.register_capability(Capability::new(
        "analytics:cross_tenant", "Cross-tenant analytics", vec![2, 1], CapabilityKind::AdminLike,
    ))?;

    let alice = acl.create_principal("alice")?;
    acl.grant(&alice, "read:tenant_a")?;
    acl.grant(&alice, "write:tenant_a")?;

    let bob = acl.create_principal("bob")?;
    acl.grant(&bob, "read:tenant_a")?;
    acl.grant(&bob, "read:tenant_b")?;
    acl.grant(&bob, "write:tenant_b")?;

    let analyst = acl.create_principal("analyst")?;
    acl.grant(&analyst, "analytics:cross_tenant")?;

    println!("=== Multi-Tenant Row-Level Security ===\n");

    let queries = [
        ("SELECT FROM tenant_a", &["read:tenant_a"][..]),
        ("INSERT INTO tenant_b", &["write:tenant_b"]),
        ("SELECT FROM all tenants", &["read:tenant_a", "read:tenant_b", "read:tenant_c"]),
        ("DELETE FROM tenant_a", &["write:tenant_a"]),
    ];

    let users = [
        ("Alice (tenant_a owner)", &alice),
        ("Bob (tenant_a reader, tenant_b owner)", &bob),
        ("Analyst (cross-tenant)", &analyst),
    ];

    for (query, caps) in &queries {
        println!("{query}:");
        for (label, principal) in &users {
            let decision = acl.check(principal, caps)?;
            match decision {
                AccessDecision::Granted { configurations, .. } => {
                    println!("  ✅ {label}: allowed ({configurations} configs)");
                }
                AccessDecision::Impossible { .. } => {
                    println!("  💀 {label}: IMPOSSIBLE (cross-tenant conflict)");
                }
                AccessDecision::Denied => println!("  ❌ {label}: denied"),
                AccessDecision::Underconstrained { .. } => {
                    println!("  ⚠️  {label}: too permissive");
                }
            }
        }
        println!();
    }

    // Stability analysis
    println!("=== Stability of Tenant Isolation ===\n");
    let report = schubert::analyze_stability(&acl, &bob)?;
    println!("Bob: {} breakpoints, {} walls", report.phase_diagram.len(), report.walls.len());

    for trust in [1.0, 0.5, 0.1] {
        let stable = schubert::stable_capabilities_at(
            &acl, &bob, schubert::TrustLevel::new(trust),
        )?;
        println!("  Trust {trust:.1}: stable = {stable:?}");
    }

    Ok(())
}
