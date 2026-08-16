# Architectural Philosophy

Schubert is built on a single architectural philosophy that runs through every module:
**the computation must be exact, but the infrastructure may be approximate.**

## The Boundary

This boundary appears in multiple places across the library:

| Module | Exact Side | Approximate Side |
|---|---|---|
| `controller.rs` | Littlewood-Richardson coefficients (integer) | Principal identities (external, opaque strings) |
| `surreal_trust.rs` | `RationalSurreal` + `EpsilonPolynomial` | Trust updates from external systems |
| `crdt.rs` | Geometric intersection (exact LR coefficient) | Eventually-consistent grant state |
| `holographic.rs` | Cosine similarity threshold (float, but bounded) | Vector encoding via FNV hash |
| `crypto.rs` | Ed25519 signature verification | Token serialization format |

## Why This Matters

Most access control systems blur this boundary. They use floating-point for trust
and accept both data staleness AND computational approximation. Schubert refuses
the computational approximation while accepting the data staleness.

The result: **you should be able to trust the computation even when you can't trust
the data.** When state eventually converges, the decision you made from that state
must be mathematically defensible.

## CRDT Staleness Gating

The CRDT module (`crdt.rs`) provides explicit controls for this boundary:

```rust
let mut state = CrdtState::new(2, 4)?;

// Set maximum allowed staleness — refuse decisions when grants are too old
state.set_max_staleness(Some(30_000)); // 30 seconds

// Check staleness
if let Some(staleness) = state.staleness_ms() {
    if staleness > 30_000 {
        println!("State is {staleness}ms stale — refusing decisions");
    }
}

// Cross-node convergence check
if !state.is_converged_with(&other_node_version) {
    println!("Not yet converged with other node");
}
```

Callers can choose: proceed with stale state (and accept eventual-consistency
consequences), or gate on freshness and refuse decisions until convergence.

## The Unanswered Question

What does it mean for an access decision to be "correct" when the trust level is
exact but the state is stale?

Consider: Node A grants Alice read access with surreal trust level 0.5 (exact
rational). Node B hasn't yet received the grant (CRDT state hasn't converged).
Alice asks Node B for read access. Node B computes the intersection: the
capability exists but Alice doesn't hold it. Decision: Denied.

The computation was exact. The data was incomplete. Was the decision wrong?

Schubert's answer: the library provides the tools to detect this situation
(`staleness_ms`, `is_converged_with`, `set_max_staleness`), but it's the
caller's choice whether to proceed. Some systems should refuse decisions
on stale data. Others should serve from whatever state they have and accept
that convergence will eventually resolve discrepancies.

## Comparison with Other IA Projects

This pattern — exact interfaces for approximate infrastructure — appears across
the Industrial Algebra ecosystem:

| Project | Exact Side | Approximate Side |
|---|---|---|
| **Minuet** | Holographic memory interfaces | Optical hardware stub |
| **Virtuoso** | Cognitive agent architecture | Module stubs |
| **Schubert** | Geometric access decisions | CRDT-distributed state |
| **Minority** | Surreal number types (Amari ecosystem) | Conway operation stubs (todo!) |

The pattern is deliberate: build the rigorous mathematical foundation first.
Ship with stubs or approximate infrastructure where necessary. The math must
be correct from day one; the infrastructure can evolve.
