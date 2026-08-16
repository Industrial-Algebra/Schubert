# Distributed CRDTs

Eventually-consistent access grants using Conflict-Free Replicated Data Types
(CRDTs). Multiple nodes can independently grant/revoke capabilities and merge.

## CrdtState

```rust
use schubert::crdt::{CrdtState, CrdtGrant, VersionVector};

let mut node_a = CrdtState::new();
let mut node_b = CrdtState::new();

// Node A grants a capability
node_a.apply(CrdtGrant::grant("alice", "read"))?;

// Node B grants a capability (concurrently)
node_b.apply(CrdtGrant::grant("alice", "write"))?;

// Merge — both grants survive
node_a.merge(&node_b)?;
assert!(node_a.has_grant("alice", "read"));
assert!(node_a.has_grant("alice", "write"));
```

## Version Vectors

Each grant carries a version vector tracking causal history:

```rust
let grant = CrdtGrant::grant("alice", "read");
println!("Version: {:?}", grant.version());
```

## Last-Write-Wins

Conflicting grants (same principal, same capability) resolve via last-write-wins:

```rust
// Node A grants, Node B revokes concurrently
let grant = CrdtGrant::grant("alice", "read");
let revoke = CrdtGrant::revoke("alice", "read");

// Merge resolves to the operation with the higher timestamp
node_a.apply(grant)?;
node_a.merge(&node_b)?; // state_b has the revoke with higher timestamp
```

## Merge Properties

- **Commutative**: `a.merge(b) == b.merge(a)`
- **Associative**: `(a.merge(b)).merge(c) == a.merge(b.merge(c))`
- **Idempotent**: `a.merge(a) == a`
