// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: AGPL-3.0-only

//! Quantitative rate limiting via Schubert intersection numbers.
//!
//! Demonstrates token-bucket rate limiting where bucket capacity is
//! determined by the Schubert intersection number from access decisions.
//!
//! Run with: `cargo run --example rate_limiter`

use schubert::{AccessController, Capability, CapabilityKind, RateLimiter};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Quantitative Rate Limiting ===\n");

    // Set up access control
    let mut acl = AccessController::new(2, 4)?;
    acl.register_capability(Capability::new(
        "read",
        "Read",
        vec![1],
        CapabilityKind::ReadLike,
    ))?;
    acl.register_capability(Capability::new(
        "write",
        "Write",
        vec![2],
        CapabilityKind::WriteLike,
    ))?;
    acl.register_capability(Capability::new(
        "admin",
        "Admin",
        vec![2, 2],
        CapabilityKind::AdminLike,
    ))?;

    let alice = acl.create_principal("alice")?;
    acl.grant(&alice, "read")?;

    let bob = acl.create_principal("bob")?;
    acl.grant(&bob, "read")?;
    acl.grant(&bob, "write")?;
    acl.grant(&bob, "admin")?;

    // Set up rate limiter: 10 tokens/sec per configuration
    let mut rl = RateLimiter::new(10.0, 1.0);

    // Configure from access decisions
    let alice_decision = acl.check(&alice, &["read"])?;
    let bob_decision = acl.check(&bob, &["admin"])?;

    rl.configure_from_decision("alice", &alice_decision)?;
    rl.configure_from_decision("bob", &bob_decision)?;

    // Alice: read = σ₁ → intersection with position → positive dimensional
    // (underconstrained, no finite config count for rate limiting)
    println!("Alice's decision: {alice_decision:?}");
    println!(
        "Alice's rate limit: {:.1} tokens",
        rl.capacity("alice").unwrap_or(0.0)
    );

    // Bob: admin = σ₂₂ → point class → 1 configuration
    println!("Bob's decision: {bob_decision:?}");
    println!(
        "Bob's rate limit: {:.1} tokens",
        rl.capacity("bob").unwrap_or(0.0)
    );

    // Simulate requests
    println!("\n=== Simulated Requests ===\n");

    for i in 0..5 {
        match rl.try_consume("bob") {
            Ok(remaining) => println!("  Request {i}: ✅ allowed ({remaining:.1} tokens left)"),
            Err(e) => println!("  Request {i}: ❌ {e}"),
        }
    }

    // Bob only has ~10 tokens (1 config * 10 base_rate) — should exhaust
    println!("\n=== After Exhaustion ===\n");
    println!(
        "Can bob consume? {}",
        if rl.can_consume("bob") { "yes" } else { "no" }
    );

    // Alice had an underconstrained result → couldn't configure
    // Demonstrate manual configuration from a known intersection number
    rl.configure_principal("carol", 4); // sigma1^4 = 2 configs in Gr(2,4)
    println!(
        "Carol (manually configured, n=4): {:.1} tokens capacity",
        rl.capacity("carol").unwrap()
    );

    // Higher intersection = more tokens
    rl.configure_principal("dave", 1); // sigma22 = 1 config
    println!(
        "Dave (n=1): {:.1} tokens capacity",
        rl.capacity("dave").unwrap()
    );

    let carol_cap = rl.capacity("carol").unwrap();
    let dave_cap = rl.capacity("dave").unwrap();
    println!(
        "\nCarol gets {:.1}x more throughput than Dave",
        carol_cap / dave_cap
    );

    Ok(())
}
