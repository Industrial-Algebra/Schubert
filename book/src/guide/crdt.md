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

## Grant Tombstones (v0.5.0)

Blanket `(principal, capability)` revocation cannot target *one specific
issuance* — and the LWW map is the wrong tool for "kill this bearer now",
because concurrent ops can resurrect it. So grant-aware revocation
([ADR-0002](https://github.com/Industrial-Algebra/Schubert/blob/develop/docs/adr/0002-grant-crdt-revocation.md))
is a **grow-only tombstone set keyed by the issuance nonce** (the signed
16-byte field every `GrantToken` has carried since v0.5.0):

```rust
use schubert::crdt::CrdtState;

let mut state = CrdtState::new(2, 4)?;

// Tombstone one bearer: pass the grant's signed nonce from the crypto layer.
state.revoke_grant(grant.nonce, "node-a", timestamp_ms)?;
assert!(state.is_grant_revoked(&grant.nonce));
```

Merge is **union**, so a tombstoned issuance can never be resurrected by any
merge order — the property federation consumers need. Renewal is unaffected:
re-issue produces a *fresh* nonce (ADR-0001), and only the old bearer dies.
Tombstones accumulate (grow-only); pruning is an operator concern. The
TypeScript `GrantCRDT` mirrors this exactly (`revokeGrant(nonceHex)` /
`isGrantRevoked(nonceHex)`), with snapshots carrying the set and readers
tolerating pre-v0.5.0 snapshots.
