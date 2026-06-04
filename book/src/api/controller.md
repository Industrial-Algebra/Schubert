# AccessController

The main entry point for all access control operations.

## Construction

```rust
use schubert::AccessController;

let mut acl = AccessController::new(2, 4)?; // Gr(2,4)
```

## Principal Management

```rust
// Create a principal
let alice = acl.create_principal("alice")?;

// Get an existing principal
let bob = acl.get_principal("bob")?;

// List all principals
let principals = acl.principals();
```

## Capability Management

```rust
use schubert::{Capability, CapabilityKind};

// Register a capability
acl.register_capability(Capability::new(
    "read:data",
    "Read data access",
    vec![1],
    CapabilityKind::ReadLike,
))?;

// List registered capabilities
let capabilities = acl.capabilities();

// Check if a capability is registered
if acl.has_capability("read:data") {
    // ...
}
```

## Grant / Revoke

```rust
acl.grant(&alice, "read:data")?;
acl.grant(&alice, "write:data")?;

// Revoke
acl.revoke(&alice, "write:data")?;

// Check what a principal holds
let held = acl.held_by(&alice);
```

## Access Check

```rust
let result = acl.check(&alice, &["read:data"])?;

match result {
    AccessDecision::Granted { configurations } => { /* ... */ }
    AccessDecision::Impossible { conflicting } => { /* ... */ }
    AccessDecision::Denied => { /* ... */ }
    AccessDecision::Underconstrained { dimension } => { /* ... */ }
}
```

## Context-Aware Check

```rust
use schubert::AccessContext;

let ctx = AccessContext {
    resource: Some("customer-data".into()),
    time_budget_ms: Some(500),
    required_trust: 0.85,
};

acl.check_with_context(&alice, &["read:data"], &ctx)?;
```

## Batch Operations (parallel feature)

```rust
let queries = vec![
    (alice.clone(), vec!["read:data"]),
    (bob.clone(), vec!["write:data"]),
];

let results = acl.check_batch(&queries)?;
```

## Computation Path

```rust
use schubert::ComputationPath;

acl.set_computation_path(ComputationPath::LR);
acl.set_computation_path(ComputationPath::Tropical);
```

## Audit Sink

```rust
use schubert::audit::InMemoryAudit;

acl.set_audit_sink(Box::new(InMemoryAudit::new()));
// Every check() call now records to the sink
```
