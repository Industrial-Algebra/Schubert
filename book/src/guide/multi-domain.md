# Multi-Domain Access

`MultiController` manages access across multiple Grassmannian domains with
cross-domain capability translation.

## Setup

```rust
use schubert::MultiController;

let mut mc = MultiController::new();

// Register domains
let rbac_domain = mc.add_domain_named(2, 4, "rbac")?;
let tenant_domain = mc.add_domain_named(3, 6, "multi-tenant")?;

// Create principal in a domain
let alice = mc.create_principal("alice", &rbac_domain)?;

// Grant capabilities within a domain
mc.grant_in_domain(&alice, "read", &rbac_domain)?;
mc.grant_in_domain(&alice, "write", &rbac_domain)?;
```

## Same-Domain Check

```rust
let result = mc.check_in_domain(&alice, &["read", "write"], &rbac_domain)?;
```

## Cross-Domain Check

Translates capabilities between Grassmannians using Schubert intersection:

```rust
// Check if RBAC read/write capabilities work in the tenant domain
let result = mc.check_cross_domain(
    &alice,
    &["read", "write"],
    &rbac_domain,     // from this domain
    &tenant_domain,   // to this domain
)?;
```

## Domain Discovery

```rust
// Find domains that accept a given partition
let domains = mc.domains_for_partition(&[1])?;

// List capabilities translatable between domains
let translatable = mc.translatable_capabilities(&rbac_domain, &tenant_domain)?;
```
