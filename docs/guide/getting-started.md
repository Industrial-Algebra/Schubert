# Getting Started with Schubert

Schubert replaces boolean allow/deny with **geometric access control**. This
guide walks you through installing, configuring, and using Schubert in your
first project.

## Installation

Add Schubert to your `Cargo.toml`:

```toml
[dependencies]
schubert = "0.1.0"
```

Schubert requires **nightly Rust** (IA ecosystem standard):

```bash
rustup toolchain install nightly
rustup default nightly
```

## Your First Access Controller

```rust
use schubert::{AccessController, Capability, CapabilityKind, AccessDecision};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a controller for Gr(2,4) — 4-dimensional policy space
    let mut acl = AccessController::new(2, 4)?;

    // Register capabilities (Schubert conditions)
    acl.register_capability(Capability::new(
        "read:data", "Read data", vec![1], CapabilityKind::ReadLike,
    ))?;
    acl.register_capability(Capability::new(
        "write:data", "Write data", vec![2], CapabilityKind::WriteLike,
    ))?;

    // Create a principal and grant capabilities
    let alice = acl.create_principal("alice")?;
    acl.grant(&alice, "read:data")?;
    acl.grant(&alice, "write:data")?;

    // Check access
    match acl.check(&alice, &["read:data", "write:data"])? {
        AccessDecision::Granted { configurations, .. } => {
            println!("Granted with {configurations} configurations");
        }
        AccessDecision::Impossible { conflicting } => {
            println!("Geometrically impossible: {conflicting:?}");
        }
        AccessDecision::Denied => println!("Access denied"),
        AccessDecision::Underconstrained { dimension } => {
            println!("Policy too permissive (dimension {dimension})");
        }
    }

    Ok(())
}
```

## Choosing a Grassmannian

| Gr(k,n) | Dimension | Use Case |
|---------|-----------|----------|
| Gr(2,4) | 4 | Standard RBAC (recommended) |
| Gr(3,6) | 9 | Complex multi-tenant |
| Gr(4,8) | 16 | Enterprise policy space |

## What's Next?

- [Concepts](concepts.md) — mathematical foundation
- [Architecture](architecture.md) — module map and data flow
- [Cookbook](cookbook.md) — integration recipes
- [Feature Flags](feature-flags.md) — enabling optional features
- [API Reference](https://docs.rs/schubert) — full rustdoc
