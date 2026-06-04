# Access Decisions

When you call `acl.check()`, Schubert returns an `AccessDecision` — not a boolean,
but a quantitative result with four variants.

## The Four Decisions

```rust
pub enum AccessDecision {
    /// Access allowed with exactly this many configurations
    Granted { configurations: usize },
    /// Conditions are geometrically incompatible (Littlewood-Richardson = 0)
    Impossible { conflicting: Vec<String> },
    /// Too many conditions for the policy space (overconstrained)
    Denied,
    /// Too few conditions — policy is loose
    Underconstrained { dimension: usize },
}
```

### Granted

The intersection number is positive. The principal can access the resource in
`configurations` distinct ways.

```rust
acl.grant(&alice, "read")?;
acl.grant(&alice, "write")?;

let result = acl.check(&alice, &["read", "write"])?;
match result {
    AccessDecision::Granted { configurations } => {
        // configurations is the Littlewood-Richardson coefficient
        // for σ₁ ∩ σ₂ in this Grassmannian
    }
    _ => {}
}
```

### Impossible

The killer feature. Individual capabilities are valid but together they're
geometrically impossible. A traditional boolean AND would approve.

```rust
// σ₂ (write) and σ₁₁ (internal audit) in Gr(2,4)
acl.grant(&principal, "write")?;
acl.grant(&principal, "internal_audit")?;

let result = acl.check(&principal, &["write", "internal_audit"])?;
// AccessDecision::Impossible { conflicting: ["write", "internal_audit"] }
```

The Littlewood-Richardson coefficient σ₂ · σ₁₁ = 0 in Gr(2,4). No subspace can
simultaneously satisfy both conditions.

### Denied

Too many independent conditions — the intersection is empty because the total
codimension exceeds the Grassmannian dimension.

```rust
// Gr(2,4) has dimension 4 — can't impose 5 independent conditions
acl.check(&principal, &["c1", "c2", "c3", "c4", "c5"])?;
// AccessDecision::Denied
```

### Underconstrained

Too few conditions — the policy doesn't pin down a specific configuration.
The remaining dimension tells you how loose the policy is.

```rust
// Only one condition in Gr(2,4) with dimension 4
// Remaining dimension = 4 - 1 = 3
acl.check(&principal, &["read"])?;
// AccessDecision::Underconstrained { dimension: 3 }
```

## Computation Paths

Schubert supports four computation engines for computing intersection numbers:

| Path | When to use |
|---|---|
| `LR` | Default — balanced performance, exact results |
| `Localization` | When you need geometric insight into *why* |
| `Tropical` | Large-scale batch operations (>1000 principals) |
| `Matroid` | When parallel evaluation is enabled |

```rust
use schubert::ComputationPath;

acl.set_computation_path(ComputationPath::Tropical);
```
