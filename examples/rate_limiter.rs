// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! Quantitative rate limiting via Schubert intersection numbers.
//!
//! Demonstrates token-bucket rate limiting where bucket capacity is
//! determined by the Schubert intersection number from access decisions.
//!
//! Run with: `cargo run --example rate_limiter`

use schubert::{
    AccessController, Capability, CapabilityId, CapabilityKind, PrincipalId, RateLimiter,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Quantitative Rate Limiting ===\n");

    // Set up access control
    let mut acl = AccessController::new(2, 4)?;
    acl.register_capability(Capability::new(
        CapabilityId::new("read").expect("valid id"),
        "Read",
        vec![1],
        CapabilityKind::ReadLike,
    ))?;
    acl.register_capability(Capability::new(
        CapabilityId::new("write").expect("valid id"),
        "Write",
        vec![2],
        CapabilityKind::WriteLike,
    ))?;
    acl.register_capability(Capability::new(
        CapabilityId::new("admin").expect("valid id"),
        "Admin",
        vec![2, 2],
        CapabilityKind::AdminLike,
    ))?;

    let alice = acl.create_principal(PrincipalId::new("alice").expect("valid id"))?;
    acl.grant(&alice, "read")?;

    let bob = acl.create_principal(PrincipalId::new("bob").expect("valid id"))?;
    acl.grant(&bob, "read")?;
    acl.grant(&bob, "write")?;
    acl.grant(&bob, "admin")?;

    // Set up rate limiter: 10 tokens/sec per configuration
    let mut rl = RateLimiter::new(10.0, 1.0);

    // Configure from access decisions
    let alice_decision = acl.check(&alice, &["read"])?;
    let bob_decision = acl.check(&bob, &["admin"])?;

    rl.configure_from_decision(
        PrincipalId::new("alice").expect("valid id"),
        &alice_decision,
    )?;
    rl.configure_from_decision(PrincipalId::new("bob").expect("valid id"), &bob_decision)?;

    // Alice: read = σ₁ → intersection with position → positive dimensional
    // (underconstrained, no finite config count for rate limiting)
    println!("Alice's decision: {alice_decision:?}");
    println!(
        "Alice's rate limit: {:.1} tokens",
        rl.capacity(PrincipalId::new("alice").expect("valid id"))
            .unwrap_or(0.0)
    );

    // Bob: admin = σ₂₂ → point class → 1 configuration
    println!("Bob's decision: {bob_decision:?}");
    println!(
        "Bob's rate limit: {:.1} tokens",
        rl.capacity(PrincipalId::new("bob").expect("valid id"))
            .unwrap_or(0.0)
    );

    // Simulate requests
    println!("\n=== Simulated Requests ===\n");

    for i in 0..5 {
        match rl.try_consume(PrincipalId::new("bob").expect("valid id")) {
            Ok(remaining) => println!("  Request {i}: ✅ allowed ({remaining:.1} tokens left)"),
            Err(e) => println!("  Request {i}: ❌ {e}"),
        }
    }

    // Bob only has ~10 tokens (1 config * 10 base_rate) — should exhaust
    println!("\n=== After Exhaustion ===\n");
    println!(
        "Can bob consume? {}",
        if rl.can_consume(PrincipalId::new("bob").expect("valid id")) {
            "yes"
        } else {
            "no"
        }
    );

    // Alice had an underconstrained result → couldn't configure
    // Demonstrate manual configuration from a known intersection number
    rl.configure_principal(PrincipalId::new("carol").expect("valid id"), 4); // sigma1^4 = 2 configs in Gr(2,4)
    println!(
        "Carol (manually configured, n=4): {:.1} tokens capacity",
        rl.capacity(PrincipalId::new("carol").expect("valid id"))
            .unwrap()
    );

    // Higher intersection = more tokens
    rl.configure_principal(PrincipalId::new("dave").expect("valid id"), 1); // sigma22 = 1 config
    println!(
        "Dave (n=1): {:.1} tokens capacity",
        rl.capacity(PrincipalId::new("dave").expect("valid id"))
            .unwrap()
    );

    let carol_cap = rl
        .capacity(PrincipalId::new("carol").expect("valid id"))
        .unwrap();
    let dave_cap = rl
        .capacity(PrincipalId::new("dave").expect("valid id"))
        .unwrap();
    println!(
        "\nCarol gets {:.1}x more throughput than Dave",
        carol_cap / dave_cap
    );

    Ok(())
}
