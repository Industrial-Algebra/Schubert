// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: AGPL-3.0-only

//! Context-aware access decisions with resource scoping and time decay.
//!
//! Run with: `cargo run --example context_aware`

use schubert::{AccessContext, AccessController, Capability, CapabilityKind};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Context-Aware Access Decisions ===\n");

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

    for doc_id in &["doc/42", "doc/99"] {
        acl.register_capability(Capability::new(
            format!("read/{doc_id}"),
            format!("Read {doc_id}"),
            vec![1],
            CapabilityKind::ReadLike,
        ))?;
    }

    let alice = acl.create_principal("alice")?;
    acl.grant(&alice, "read")?;
    acl.grant(&alice, "read/doc/42")?;

    let bob = acl.create_principal("bob")?;
    acl.grant(&bob, "read")?;

    println!("=== Resource Scoping ===\n");
    let ctx = AccessContext::for_resource("doc/42");
    let d = acl.check_with_context(&alice, &["read"], &ctx)?;
    println!("alice → read (resource=doc/42): {d:?} (includes scoped cap)");

    let ctx = AccessContext::for_resource("doc/42");
    let d = acl.check_with_context(&bob, &["read"], &ctx)?;
    println!("bob   → read (resource=doc/42): {d:?} (no scoped cap held)");

    let ctx = AccessContext::for_resource("doc/99");
    let d = acl.check_with_context(&alice, &["read"], &ctx)?;
    println!("alice → read (resource=doc/99): {d:?} (scoped cap not held)");

    println!("\n=== Time-Aware Trust ===\n");
    let fresh_time = acl.principal(&alice).unwrap().created_at;
    let one_year = fresh_time + 31_536_000_000u64;

    let ctx = AccessContext::at_time(fresh_time);
    let d = acl.check_with_context(&alice, &["admin"], &ctx)?;
    println!("admin at t=0 (fresh):           {d:?}");

    let ctx = AccessContext::at_time(one_year);
    let d = acl.check_with_context(&alice, &["admin"], &ctx)?;
    println!("admin at t=1yr:                 {d:?}");

    println!("\n=== Empty Context = Standard Check ===\n");
    let standard = acl.check(&alice, &["read"])?;
    let with_empty = acl.check_with_context(&alice, &["read"], &AccessContext::empty())?;
    assert_eq!(standard, with_empty);
    println!("✅ Empty context matches standard check");

    Ok(())
}
