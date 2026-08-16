# Capability & Principal

## Capability

A Schubert condition with a partition, kind, and label.

```rust
use schubert::{Capability, CapabilityKind};

let cap = Capability::new(
    "read:data",        // unique ID
    "Read data access", // human label
    vec![1],            // partition (Schubert condition)
    CapabilityKind::ReadLike,
);
```

### Fields

| Field | Type | Description |
|---|---|---|
| `id` | `&str` | Unique capability identifier |
| `label` | `&str` | Human-readable description |
| `partition` | `Vec<usize>` | Schubert partition (weakly decreasing) |
| `kind` | `CapabilityKind` | Semantic category affecting trust sensitivity |
| `expires_at` | `Option<u64>` | Optional expiry timestamp (milliseconds) |

### Methods

```rust
// Temporal capabilities
let temp = cap.with_expiry(now + 3_600_000); // 1 hour
let remaining = temp.time_remaining_at(check_time);
let is_expired = temp.is_expired_at(check_time);

// Codimension (sum of partition entries)
let codim = cap.codimension(); // 1 for [1], 3 for [2,1]
```

## CapabilityKind

```rust
pub enum CapabilityKind {
    ReadLike,   // Low trust sensitivity
    WriteLike,  // Medium trust sensitivity
    AdminLike,  // High trust sensitivity
    Custom,     // Application-defined
}
```

## PrincipalId

An opaque identity wrapper. Schubert never authenticates — identity is
provided by your external auth system.

```rust
use schubert::PrincipalId;

let alice = PrincipalId::new("alice");
let from_jwt = PrincipalId::new(jwt_claims.sub);
```

`PrincipalId` implements `Clone`, `Eq`, `Hash`, `Debug`, and with `serde`:
`Serialize`/`Deserialize`.
