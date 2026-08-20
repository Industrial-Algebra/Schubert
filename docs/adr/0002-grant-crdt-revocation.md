# ADR-0002: Grant-aware CRDT revocation — tombstone registry keyed by issuance nonce

**Status:** Proposed (2026-08-17). Scope: Roadmap #20.2 (v0.5.0).
**Consumers:** Wallace (multi-participant sessions — revoke a session grant everywhere),
Ijima (peer revocation across replicas), Dominic (federation-wide revocation converging
without a central controller).
**Predecessor:** ADR-0001 (expiry & nonce — this ADR depends on its rule 4:
*renewal = re-issue*, enabled by the per-issuance nonce).

## Decision (proposed)

**A grow-only tombstone set keyed by the #20.1 issuance nonce.** `CrdtGrant` is
unchanged; `CrdtState` (Rust) and `GrantCRDT` (tsukoshi) each gain a
`revoked_grants` set merged by **union**:

- `revoke_grant(nonce, node)` — tombstone one specific issuance; idempotent.
- `is_grant_revoked(nonce)` — the read path.
- Merge = set union. Union is commutative, associative, and idempotent trivially —
  a textbook G-Set — so convergence needs no version vectors, tiebreaks, or LWW.

## Why not upgrade `CrdtGrant` to carry a grant id

Two independent reasons, either sufficient:

1. **Resurrection.** The tsukoshi `GrantCRDT` resolves concurrent ops
   **add-wins** (grant beats revoke, `pickWinner`). A revoke carried on the
   grant entry itself can therefore be *beaten* by a concurrent re-grant of the
   same grant id — the revoked issuance comes back to life. For incident-grade
   "kill this token now" semantics, that is disqualifying. A G-Set tombstone
   cannot be resurrected by any merge order, by construction.
2. **Semantic perturbation.** Keying entries by `(principal, capability,
grant_id)` turns the well-tested single-entry-per-(p,c) LWW map into a
multi-map, forcing every `holds`/`check`/`active_grants` path to aggregate over
multiple concurrent issuances — churn out of proportion to the need.

## Why the nonce is the key

The #20.1 nonce is 16 random bytes, **signed**, and distinct per issuance —
already the canonical grant identifier in the token itself. A content hash of
the token bytes would also work but adds nothing (the nonce is inside the
signed content). Critically, per ADR-0001 rule 4, renewal is *re-issue*: the
rotated grant carries a **fresh nonce**, so tombstoning the old issuance never
kills its replacement.

## Composition

The tombstone is the third OR-branch of the access predicate (ADR-0001 rule 5,
extended):

> access = valid signature **and** not expired **and** not tombstoned (this
> ADR) **and** not blanket-revoked at the (principal, capability) level.

Blanket `(p,c)` revocation (`revoke`) remains the policy-level deprovisioning
path; grant tombstones target specific issued bearers. Rust `CrdtState` stays
free of `crypto` types — callers pass `&grant.nonce` from the crypto layer.

## Consequences

- **Unresurrectable, order-free convergence** for grant-level revocation — the
  property federation consumers actually need.
- **Unbounded growth**: tombstones accumulate. Pruning is an operator concern
  (safe only once every replica has converged past the issuance window);
  deferred — a `prune` API can follow if size ever matters.
- Serialization (tsukoshi `toJSON`/`fromJSON`) carries the set; readers
  tolerate snapshots from before this ADR (missing field = empty set).

## References

- Roadmap #20 (this repo) — sub-item 2, which posed the either/or this ADR resolves.
- ADR-0001 — nonce semantics; renewal = re-issue; revocation composes as OR.
- tsukoshi `grant-crdt.ts` `pickWinner` — the add-wins rule motivating the G-Set.
