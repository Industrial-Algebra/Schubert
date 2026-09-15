# Capabilities as Schubert Conditions

Every capability in Schubert is a Schubert condition — a geometric constraint with a
partition, a kind, and a label.

## Defining a Capability

```rust
use schubert::{Capability, CapabilityId, CapabilityKind};

let read = Capability::new(
    CapabilityId::new("read:data").expect("valid id"),        // unique ID
    "Read data access", // human-readable label
    vec![1],            // partition: σ₁ (codimension 1)
    CapabilityKind::ReadLike,
);

let write = Capability::new(
    CapabilityId::new("write:data").expect("valid id"),
    "Write data access",
    vec![2],             // partition: σ₂ (codimension 2)
    CapabilityKind::WriteLike,
);

let admin = Capability::new(
    CapabilityId::new("admin").expect("valid id"),
    "Full administrative access",
    vec![2, 2],          // partition: σ₂₂ (codimension 4, point class)
    CapabilityKind::AdminLike,
);
```

## CapabilityKind

The `CapabilityKind` affects how capabilities behave under trust degradation:

| Kind | Trust Sensitivity | Examples |
|---|---|---|
| `ReadLike` | Low — stable even at low trust | Read, list, view |
| `WriteLike` | Medium — degrades at moderate trust | Write, update, delete |
| `AdminLike` | High — degrades rapidly under trust loss | Admin, manage, configure |
| `Custom` | Application-defined | Any custom semantic |

Higher-codimension AdminLike capabilities are the first to become unstable as trust
erodes. This models the real-world principle that powerful capabilities should require
more trust.

## Partition Design

Partitions determine how capabilities interact. Key rules:

- **Partitions must be weakly decreasing**: `[2,1]` is valid, `[1,2]` is not
- **Codimension = sum of entries**: `[2,1]` has codimension 3
- **One element = one row**: `[1]` is a single-row condition
- **Equal partitions = same restriction**: two capabilities with `[1]` are equivalent
  in their geometric constraint

## Temporal Capabilities

Capabilities can have an expiry time:

```rust
let temp = Capability::new(CapabilityId::new("temp").expect("valid id"), "Temporary access", vec![1], CapabilityKind::ReadLike)
    .with_expiry(now + 3_600_000); // 1 hour from now

acl.register_capability(temp)?;
acl.grant(&principal, "temp")?;

// Later:
acl.check_temporal(&principal, &["temp"], now)?;       // OK
acl.check_temporal(&principal, &["temp"], later)?;      // Denied
```

## Registration and Grants

Capabilities must be registered with the controller before they can be granted:

```rust
// 1. Register
acl.register_capability(read)?;

// 2. Grant to principal
acl.grant(&alice, "read:data")?;

// 3. Check
acl.check(&alice, &["read:data"])?;
```

Registration defines the capability's geometry. Granting assigns it to a principal.
Checking evaluates the intersection.
