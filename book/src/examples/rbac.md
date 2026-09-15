# Role-Based Access Control (RBAC)

Traditional RBAC with quantitative access decisions.

**Source**: `examples/rbac.rs`

## Setup

```rust
use schubert::{AccessController, Capability, CapabilityKind, AccessDecision};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut acl = AccessController::new(2, 4)?;
```

## Capabilities

```rust
    acl.register_capability(Capability::new(
        CapabilityId::new("read").expect("valid id"), "Read access", vec![1], CapabilityKind::ReadLike,
    ))?;
    acl.register_capability(Capability::new(
        CapabilityId::new("write").expect("valid id"), "Write access", vec![2], CapabilityKind::WriteLike,
    ))?;
    acl.register_capability(Capability::new(
        CapabilityId::new("admin").expect("valid id"), "Admin access", vec![2, 1], CapabilityKind::AdminLike,
    ))?;
```

## Principals and Grants

```rust
    let alice = acl.create_principal(PrincipalId::new("alice").expect("valid id"))?;
    acl.grant(&alice, "read")?;
    acl.grant(&alice, "write")?;

    let bob = acl.create_principal(PrincipalId::new("bob").expect("valid id"))?;
    acl.grant(&bob, "read")?;
```

## Access Checks

```rust
    // Alice can read and write
    match acl.check(&alice, &["read", "write"])? {
        AccessDecision::Granted { configurations } => {
            println!("Alice: granted with {configurations} configurations");
        }
        _ => unreachable!(),
    }

    // Bob cannot write
    match acl.check(&bob, &["read", "write"])? {
        AccessDecision::Granted { .. } => unreachable!(),
        _ => println!("Bob: cannot read and write"),
    }

    Ok(())
}
```

## Key Takeaway

RBAC is the simplest pattern — define roles as capability sets, grant them to
principals, and check. The quantitative nature of Schubert shines when roles
overlap or conflict.
