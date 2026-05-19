// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: AGPL-3.0-only

//! Policy-driven access control from TOML.
//!
//! Demonstrates loading a policy from a TOML file, performing access checks,
//! and exporting the policy back to TOML.
//!
//! Run with: `cargo run --example policy_loader --features policy`

use schubert::{AccessController, AccessDecision, PrincipalId};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Define a policy inline
    let policy = r#"
[grassmannian]
k = 2
n = 4

[capabilities.read_data]
partition = [1]
kind = "ReadLike"
label = "Read data"
description = "Read-only access to data records"

[capabilities.write_data]
partition = [2]
kind = "WriteLike"
label = "Write data"
description = "Write access to data records"

[capabilities.manage_users]
partition = [2, 1]
kind = "AdminLike"
label = "Manage users"
description = "Create, update, and delete users"

[capabilities.admin_all]
partition = [2, 2]
kind = "AdminLike"
label = "Full admin"
description = "Full administrative access"

[principals.alice]
grants = ["read_data", "write_data"]

[principals.bob]
grants = ["read_data"]

[principals.carol]
grants = ["read_data", "manage_users"]

[principals.dave]
grants = ["admin_all"]
"#;

    println!("=== Policy-Driven Access Control ===\n");

    // Load the policy — validates all partitions against Gr(2,4)
    let acl = AccessController::from_policy_toml(policy)?;
    println!("✅ Policy loaded and validated for Gr(2,4)");
    println!("   Capabilities: {}", acl.capabilities().count());
    println!();

    // Check each principal's access
    let checks = [
        ("alice", &["read_data", "write_data"] as &[&str]),
        ("alice", &["read_data"]),
        ("bob", &["write_data"]), // bob doesn't have write
        ("carol", &["read_data", "manage_users"]),
        ("dave", &["admin_all"]),
        ("bob", &["admin_all"]), // bob tried to admin
    ];

    println!("Access checks:");
    for (principal, required) in &checks {
        let pid = PrincipalId::new(*principal);
        let decision = acl.check(&pid, required)?;
        let status = match &decision {
            AccessDecision::Granted {
                configurations,
                path,
            } => {
                format!("✅ GRANTED ({configurations} configs via {path:?})")
            }
            AccessDecision::Impossible { conflicting } => {
                let ids: Vec<_> = conflicting.iter().map(|c| c.as_str()).collect();
                format!("💀 IMPOSSIBLE — conflicting: {ids:?}")
            }
            AccessDecision::Denied => "❌ DENIED".to_string(),
            AccessDecision::Underconstrained { dimension } => {
                format!("⚠️  UNDERCONSTRAINED (dim {dimension})")
            }
        };
        let reqs = required.to_vec();
        println!("  {principal:>6} → {reqs:?}: {status}");
    }

    // Roundtrip: export to TOML and re-import
    println!("\n=== Policy Roundtrip ===\n");
    let exported = acl.to_policy_toml()?;
    let reimported = AccessController::from_policy_toml(&exported)?;

    // Verify the reimported controller produces the same decisions
    let original_decision = acl.check(&PrincipalId::new("alice"), &["read_data"])?;
    let roundtrip_decision = reimported.check(&PrincipalId::new("alice"), &["read_data"])?;
    assert_eq!(original_decision, roundtrip_decision);
    println!("✅ Policy roundtrip: decisions match after export → import");

    println!("\n=== Exported Policy (first 200 chars) ===\n");
    println!("{}...", &exported[..exported.len().min(200)]);

    Ok(())
}
