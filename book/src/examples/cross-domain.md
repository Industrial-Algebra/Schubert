# Cross-Domain Access

Capability translation between Grassmannians.

**Source**: `examples/cross_domain.rs`

## Pattern

```rust
let mut mc = MultiController::new();

// Two domains with different policy spaces
let rbac = mc.add_domain_named(2, 4, "rbac")?;       // dim 4
let tenant = mc.add_domain_named(3, 6, "multi-tenant")?; // dim 9

let alice = mc.create_principal("alice", &rbac)?;
mc.grant_in_domain(&alice, "read", &rbac)?;
mc.grant_in_domain(&alice, "write", &rbac)?;

// Check if RBAC capabilities translate to tenant domain
let result = mc.check_cross_domain(
    &alice, &["read", "write"], &rbac, &tenant
)?;
```

## Key Takeaway

Capabilities aren't globally meaningful — they live in a specific Grassmannian.
`check_cross_domain()` uses Schubert intersection to determine if a capability
set in one domain is valid in another.
