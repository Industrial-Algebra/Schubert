# Composition and Composability

Schubert supports **operadic composition** — combining capabilities across principals to
model service chains, delegation, and capability translation.

## Operadic Composition

Two capabilities are **composable** if their Schubert intersection is non-empty. The
result includes a **multiplicity** — how many configurations survive the composition:

```rust
use schubert::compose;

let result = compose(&acl, &producer, "output", &consumer, "input")?;

match result {
    CompositionResult::Composable { multiplicity } => {
        println!("{multiplicity} configurations survive composition");
    }
    CompositionResult::NotComposable { reason } => {
        println!("Cannot compose: {reason}");
    }
}
```

## Service Chain Model

Composition models real-world service chains:

```
Service A (produces "report")  ─┐
                                ├─► Compose? Multiplicity?
Service B (consumes "report")  ─┘
```

If Service A's output capability and Service B's input capability are composable
with multiplicity > 0, the service chain is valid.

## Mathematical Properties

- **Commutativity**: Grant order doesn't affect composition result
- **Associativity**: `(a ∘ b) ∘ c = a ∘ (b ∘ c)`
- **Identity**: Grant then revoke = no net change
- **Impossibility is symmetric**: If `a ∘ b` is impossible, `b ∘ a` is too

## Cross-Domain Composition

For multi-Grassmannian setups, use `MultiController`:

```rust
use schubert::MultiController;

let mut mc = MultiController::new();
let rbac = mc.add_domain(2, 4)?;     // RBAC domain
let tenant = mc.add_domain(3, 6)?;   // Multi-tenant domain

mc.create_principal("alice", &rbac)?;
mc.grant_in_domain(&alice, "read", &rbac)?;

// Check if an RBAC capability works in the tenant domain:
mc.check_cross_domain(&alice, &["read"], &rbac, &tenant)?;
```

Cross-domain checks use Schubert intersection to determine if capabilities
translate between Grassmannians.

## Checking Composability

Before attempting composition, check if capabilities are composable:

```rust
if are_composable(&acl, "read", "write")? {
    let result = compose(&acl, &alice, "read", &bob, "write")?;
    // ...
}
```

The `are_composable()` check is cheaper than full composition — it only checks
whether the Littlewood-Richardson coefficient is non-zero.
