# Getting Started

## Installation

Add Schubert to your `Cargo.toml`:

```toml
[dependencies]
schubert = "0.1"
```

Schubert requires **nightly Rust** (the Industrial Algebra ecosystem standard):

```bash
rustup toolchain install nightly
rustup default nightly
```

## Your First Access Controller

```rust
use schubert::{
    AccessController, Capability, CapabilityKind,
    AccessDecision, PrincipalId,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a controller for Gr(2,4) — the standard RBAC space
    let mut acl = AccessController::new(2, 4)?;

    // Register capabilities (Schubert conditions)
    acl.register_capability(Capability::new(
        "read:data",
        "Read data access",
        vec![1],               // σ₁ — codimension 1
        CapabilityKind::ReadLike,
    ))?;
    acl.register_capability(Capability::new(
        "write:data",
        "Write data access",
        vec![2],               // σ₂ — codimension 2
        CapabilityKind::WriteLike,
    ))?;

    // Create a principal and grant capabilities
    let alice = acl.create_principal("alice")?;
    acl.grant(&alice, "read:data")?;
    acl.grant(&alice, "write:data")?;

    // Check access — returns a quantitative decision
    match acl.check(&alice, &["read:data", "write:data"])? {
        AccessDecision::Granted { configurations } => {
            println!("Granted with {configurations} configurations");
        }
        AccessDecision::Impossible { conflicting } => {
            println!("Geometrically impossible: {conflicting:?}");
        }
        AccessDecision::Denied => {
            println!("Access denied (overconstrained)");
        }
        AccessDecision::Underconstrained { dimension } => {
            println!("Policy too permissive (dimension {dimension})");
        }
    }

    Ok(())
}
```

## Understanding the Output

| Decision | Meaning |
|---|---|
| `Granted { configurations: n }` | Access allowed in exactly n ways |
| `Impossible { conflicting }` | Conditions are geometrically incompatible |
| `Denied` | Too many conditions for the policy space |
| `Underconstrained { dimension }` | Not enough conditions (policy is loose) |

## Choosing a Grassmannian

| Gr(k,n) | Dimension k(n−k) | Use Case |
|---|---|---|
| Gr(2,4) | 4 | Standard RBAC (recommended starting point) |
| Gr(3,6) | 9 | Complex multi-tenant policies |
| Gr(4,8) | 16 | Enterprise-scale policy space |

Larger Grassmannians support more distinct capabilities but have higher computational cost.

## Next Steps

- [Mathematical Foundation](./concepts/math.md) — understand the geometry
- [Capabilities as Schubert Conditions](./concepts/capabilities.md) — designing capabilities
- [Feature Flags](./guide/feature-flags.md) — enabling optional features
- [Installation & Configuration](./guide/installation.md) — production setup
