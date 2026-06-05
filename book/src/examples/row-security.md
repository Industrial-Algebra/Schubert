# Row-Level Security

Tenant-scoped capabilities for database row-level security.

**Source**: `examples/row_security.rs`

## Pattern

Create tenant-specific capabilities and check cross-tenant access for
geometric impossibility detection:

```rust
// Tenant-scoped capabilities
acl.register_capability(Capability::new(
    "read:tenant_a", "Read tenant A", vec![1], ReadLike,
))?;
acl.register_capability(Capability::new(
    "read:tenant_b", "Read tenant B", vec![1], ReadLike,
))?;
acl.register_capability(Capability::new(
    "read:tenant_c", "Read tenant C", vec![1], ReadLike,
))?;

// Multi-tenant principal
acl.grant(&principal, "read:tenant_a")?;
acl.grant(&principal, "read:tenant_b")?;

// Three tenant reads in Gr(2,4) — too many conditions
let result = acl.check(&principal, &[
    "read:tenant_a", "read:tenant_b", "read:tenant_c",
])?;
// AccessDecision::Denied (overconstrained)
```

## Key Takeaway

Cross-tenant access patterns that a boolean system would approve are caught
as overconstrained by Schubert's geometric analysis.
