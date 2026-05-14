//! RBAC example: Kubernetes-style role-based access control.
//!
//! Models four Kubernetes-style roles as Schubert capabilities
//! with quantitative access checking in Gr(2,4).
//!
//! Roles: viewer (σ₁), editor (σ₁+σ₂), operator (σ₁+σ₂₁), admin (σ₂₂)

use schubert::{AccessController, AccessDecision, Capability, CapabilityKind};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut acl = AccessController::new(2, 4)?;

    acl.register_capability(Capability::new(
        "read:pods", "Read pods", vec![1], CapabilityKind::ReadLike,
    ))?;
    acl.register_capability(Capability::new(
        "write:pods", "Write pods", vec![2], CapabilityKind::WriteLike,
    ))?;
    acl.register_capability(Capability::new(
        "manage:deployments", "Manage deployments", vec![2, 1], CapabilityKind::AdminLike,
    ))?;
    acl.register_capability(Capability::new(
        "admin:*", "Full admin", vec![2, 2], CapabilityKind::AdminLike,
    ))?;

    let alice = acl.create_principal("alice")?;
    acl.grant(&alice, "read:pods")?;

    let bob = acl.create_principal("bob")?;
    acl.grant(&bob, "read:pods")?;
    acl.grant(&bob, "write:pods")?;

    let carol = acl.create_principal("carol")?;
    acl.grant(&carol, "read:pods")?;
    acl.grant(&carol, "manage:deployments")?;

    let dave = acl.create_principal("dave")?;
    acl.grant(&dave, "admin:*")?;

    println!("=== Kubernetes RBAC via Schubert Access Control ===\n");

    let checks = [
        ("alice (viewer)", &alice, &["read:pods"][..]),
        ("bob (editor)", &bob, &["read:pods", "write:pods"]),
        ("carol (operator)", &carol, &["read:pods", "manage:deployments"]),
        ("dave (admin)", &dave, &["admin:*"]),
        ("bob → admin", &bob, &["admin:*"]),
        ("alice → write", &alice, &["write:pods"]),
    ];

    for (label, principal, required) in &checks {
        let decision = acl.check(principal, required)?;
        print!("{label}: ");
        match decision {
            AccessDecision::Granted { configurations, path } => {
                println!("✅ GRANTED ({configurations} configs via {path:?})");
            }
            AccessDecision::Impossible { conflicting } => {
                let ids: Vec<_> = conflicting.iter().map(|c| c.as_str()).collect();
                println!("❌ IMPOSSIBLE — conflicting: {ids:?}");
            }
            AccessDecision::Denied => println!("❌ DENIED"),
            AccessDecision::Underconstrained { dimension } => {
                println!("⚠️  UNDERCONSTRAINED (dim {dimension})");
            }
        }
    }

    println!("\n=== Stability Analysis ===\n");

    for (label, principal) in [("bob", &bob), ("carol", &carol), ("dave", &dave)] {
        let report = schubert::analyze_stability(&acl, principal)?;
        println!("{label}: {} breakpoints, {} walls", report.phase_diagram.len(), report.walls.len());

        for trust in [1.0, 0.8, 0.5, 0.3, 0.1] {
            let stable = schubert::stable_capabilities_at(
                &acl, principal, schubert::TrustLevel::new(trust),
            )?;
            println!("  Trust {trust:.1}: stable = {stable:?}");
        }
        println!();
    }

    Ok(())
}
