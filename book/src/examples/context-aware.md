# Context-Aware Decisions

Resource-scoped, time-aware, trust-gated access checks.

**Source**: `examples/context_aware.rs`

## Pattern

```rust
use schubert::AccessContext;

// Time-critical operation with high trust requirement
let ctx = AccessContext {
    resource: Some("/api/critical".into()),
    time_budget_ms: Some(100),
    required_trust: 0.95,
};

let result = acl.check_with_context(&alice, &["admin"], &ctx)?;

// Low-trust read with generous time budget
let ctx = AccessContext {
    resource: Some("/api/public".into()),
    time_budget_ms: Some(5000),
    required_trust: 0.3,
};

let result = acl.check_with_context(&alice, &["read"], &ctx)?;
```

## Key Takeaway

Not all access checks are equal. High-trust, time-critical operations should
be more restrictive — `AccessContext` captures all three dimensions
(resource, time, trust) in one struct.
