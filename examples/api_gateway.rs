// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: AGPL-3.0-only

//! API Gateway example: OAuth scope intersection with conflict detection.
//!
//! Demonstrates geometrically incompatible scope combinations that
//! traditional boolean AND checks would silently approve.

use schubert::{AccessController, AccessDecision, Capability, CapabilityKind};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut acl = AccessController::new(2, 4)?;

    acl.register_capability(Capability::new(
        "read:profile",
        "Read profile",
        vec![1],
        CapabilityKind::ReadLike,
    ))?;
    acl.register_capability(Capability::new(
        "read:email",
        "Read email",
        vec![1],
        CapabilityKind::ReadLike,
    ))?;
    acl.register_capability(Capability::new(
        "write:posts",
        "Write posts",
        vec![2],
        CapabilityKind::WriteLike,
    ))?;
    acl.register_capability(Capability::new(
        "admin:users",
        "Admin users",
        vec![2, 1],
        CapabilityKind::AdminLike,
    ))?;
    // σ₁₁ — geometrically incompatible with σ₂ in Gr(2,4)
    acl.register_capability(Capability::new(
        "restricted:internal",
        "Internal only",
        vec![1, 1],
        CapabilityKind::WriteLike,
    ))?;

    let user = acl.create_principal("user_token")?;
    acl.grant(&user, "read:profile")?;
    acl.grant(&user, "read:email")?;

    let power = acl.create_principal("power_token")?;
    acl.grant(&power, "read:profile")?;
    acl.grant(&power, "write:posts")?;

    let admin = acl.create_principal("admin_token")?;
    acl.grant(&admin, "admin:users")?;

    let bad = acl.create_principal("bad_token")?;
    acl.grant(&bad, "write:posts")?; // σ₂
    acl.grant(&bad, "restricted:internal")?; // σ₁₁ — conflicts!

    println!("=== API Gateway Scope Validation ===\n");

    let endpoints = [
        ("GET /profile", &["read:profile"][..]),
        ("POST /posts", &["write:posts"]),
        ("DELETE /users/:id", &["admin:users"]),
        (
            "GET /internal/dashboard",
            &["write:posts", "restricted:internal"],
        ),
    ];

    let tokens = [
        ("User token", &user),
        ("Power token", &power),
        ("Admin token", &admin),
        ("Bad token", &bad),
    ];

    for (endpoint, scopes) in &endpoints {
        println!("{endpoint}:");
        for (label, token) in &tokens {
            let decision = acl.check(token, scopes)?;
            let symbol = match &decision {
                AccessDecision::Granted { .. } => "✅",
                AccessDecision::Impossible { .. } => "💀",
                AccessDecision::Denied => "❌",
                AccessDecision::Underconstrained { .. } => "⚠️",
            };
            match &decision {
                AccessDecision::Granted {
                    configurations,
                    path,
                } => {
                    println!("  {symbol} {label}: GRANTED ({configurations} via {path:?})");
                }
                AccessDecision::Impossible { conflicting } => {
                    let ids: Vec<_> = conflicting.iter().map(|c| c.as_str()).collect();
                    println!("  {symbol} {label}: IMPOSSIBLE — {ids:?} geometrically incompatible");
                }
                AccessDecision::Denied => println!("  {symbol} {label}: DENIED"),
                AccessDecision::Underconstrained { dimension } => {
                    println!("  {symbol} {label}: UNDERCONSTRAINED (dim {dimension})");
                }
            }
        }
        println!();
    }

    println!("=== Key Insight ===");
    println!("'write:posts' (σ₂) and 'restricted:internal' (σ₁₁) are");
    println!("individually valid but geometrically incompatible in Gr(2,4).");
    println!("A traditional boolean AND would approve. Schubert catches it.");

    Ok(())
}
