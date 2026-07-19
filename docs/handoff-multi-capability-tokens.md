# Implementation Spec: Multi-Capability Tokens (Grants)

> **Status:** Implementation spec for Schubert v0.4.0. Driven by Ijima's pi
> integration (env-bundle workaround, not blocked) and Dominic's federation
> orchestration. Ijima 0.2.0 is the target consumer — we have time to do
> this right.
> Author: Ijima session, 2026-07-12. Revised with geometric-containment
> design resolution.

## The problem

Schubert tokens are **one-capability-each**: `CapabilityToken` carries a
single `CapabilityId`, and `issue_batch` produces *N separate* one-cap
tokens, not one token holding a set. Real consumers need many capabilities
at once, and the one-cap model forces them to juggle a bundle of tokens:

- **Ijima pi integration** (`integrations/pi/`): the extension makes
  interleaved calls needing `memory:read`, `memory:write`, `knowledge:read`,
  `knowledge:write`, `mining:trigger`, `mining:review` — currently an
  **env-bundle of 4–6 one-cap tokens**, picked per call. Operationally
  clunky: every workstation issues + configures a token bundle, and the
  extension carries selection logic.
- **Dominic** (Anima meta-orchestrator): will hold a federation-wide grant
  (routing, domain delegation, `trust:override`, …) — exactly the same pain
  at orchestration scale.

The friction is cross-cutting; fixing it in Schubert unblocks both.

## The ask

A **single token carrying a capability set** (a "grant"): one principal,
many capabilities, signed once, verifiable per-capability. Concretely:

- A `Grant` carries `capabilities: Vec<CapabilityId>` — the granted set.
- `verify(grant, required_cap)` succeeds iff `required_cap` is geometrically
  implied by the granted set (see §Design below).
- A singleton grant (`[cap]`) is exactly today's one-cap token —
  **backward-compatible**.

## Design — Geometric Containment via the Partition Lattice

### Why not set membership?

The handoff originally listed "set membership vs. geometric containment" as
an open question. The answer is **geometric containment**, because within a
single Grassmannian it is almost free — and it makes the admin/imply
invariant automatic rather than hardcoded.

### The partition order

Schubert variety containment on Gr(k,n) is **component-wise partition
comparison**:

```text
σ_λ ⊇ σ_μ  ⟺  λ_i ≥ μ_i  for all i
```

This is an O(k) integer comparison per capability. No intersection numbers,
no LR coefficients, no Grassmannian machinery at verification time.

### What falls out for free

**Write implies read.** Holding σ₂ (write, codim 2) automatically satisfies
σ₁ (read, codim 1), because [2] ≥ [1] component-wise. The grant of "write"
geometrically contains the grant of "read." No special-casing.

**Admin implies everything.** σ₄₄₄₄ is the maximum partition — every
partition is ≤ it component-wise. An admin grant satisfies every capability.
The "admin implies all" invariant falls out of the partition lattice, not a
hardcoded string comparison.

**Rate-limit bucket.** The grant's effective permissiveness is the **minimum
codimension** among granted capabilities (the widest variety in the union).
This is geometrically correct: the union's dimension = max component
dimension = min component codimension.

### The verification algorithm

```text
may(grant: Vec<Partition>, cap: Partition) -> bool:
    for λ in grant:
        if cap ≤ λ component-wise:
            return true
    return false
```

The signing surface is `principal || sorted(grant) || issuer_key`. The
verification is partition comparison. No set-membership approximation, no
future migration.

### Backward compatibility

A singleton grant `[cap]` is exactly today's one-cap `CapabilityToken`:
- `may([λ], μ)` returns true iff μ ≤ λ — same as the current single-cap
  check.
- The wire format for a singleton grant is the same size as today's token.

## Design Decisions (Resolved)

### 1. Signing surface

**Decision:** Sign over `principal || sorted(canonical_grant) || issuer_key`.

The capability list is sorted by partition (component-wise) and encoded
canonically before signing. This ensures that two grants with the same
capabilities in different order produce the same signature — no ambiguity
in verification.

### 2. `may(cap)` semantics

**Decision:** Geometric containment via the partition lattice.

`may(grant, cap)` returns true iff any granted partition λ satisfies
`cap ≤ λ` component-wise. This is NOT set membership — a grant of {σ₂}
(write) implies σ₁ (read) even though σ₁ is not in the set.

Set membership would require the caller to enumerate every implied
capability at issuance time. Geometric containment derives implications
automatically from the partition order. The only cost is an O(k) comparison
per granted capability at verification time.

### 3. Rate-limit bucket

**Decision:** Key on the **minimum codimension** among granted capabilities.

The grant's effective permissiveness is its widest component — the granted
capability with the lowest codimension. For Ijima's grant
`{memory:read (σ₁), memory:write (σ₂), admin (σ₄₄₄₄)}`, the rate-limit key
is codim(σ₁) = 1 (the read capability is the most permissive).

Rationale: the union of Schubert varieties V_grant = σ₁ ∪ σ₂ ∪ σ₄₄₄₄ has
dimension = max(dim) = dim(σ₁). The rate limit should reflect the widest
access path.

### 4. Wire format

**Decision:** Canonical capability-id list, length-prefixed and base64-encoded.

The bearer token carries the grant as a list of `CapabilityId` strings,
sorted canonically. A bitset over the vocabulary is a premature optimization
— the vocabulary is consumer-specific (Ijima has 8 capabilities, Dominic may
have more), and a string list is debuggable and self-describing.

This supersedes the consumer-specific wire formats (Ijima's 80-line custom
encoder) and pairs with v0.4.0 item #16.1 (`CapabilityToken::to_bytes()` /
`from_bytes()`).

### 5. Admin/imply

**Decision:** Falls out of the partition lattice — no special-casing.

σ₄₄₄₄ is the maximum partition in the component-wise order on Gr(4,8).
Every partition is ≤ [4,4,4,4] component-wise. Therefore `may(grant, cap)`
returns true for any `cap` when the grant contains the admin partition.
No hardcoded `ADMIN` constant, no string comparison — pure geometry.

### 6. Issuance API

**Decision:**

```rust
impl CapabilityIssuer {
    /// Issue a grant token carrying multiple capabilities.
    pub fn issue_grant(
        &self,
        principal: impl Into<PrincipalId>,
        capabilities: impl IntoIterator<Item = impl Into<CapabilityId>>,
    ) -> Result<GrantToken>;

    /// Issue a single-capability token (backward-compatible).
    /// Equivalent to issue_grant(principal, [capability]).
    pub fn issue(
        &self,
        principal: impl Into<PrincipalId>,
        capability: impl AsRef<str>,
    ) -> Result<CapabilityToken>;
}
```

`issue_batch` (N separate one-cap tokens) is retained for cases where the
consumer genuinely wants separate tokens, but `issue_grant` is the primary
API.

## Consumer Acceptance (What Unblocks Ijima/Dominic)

- Ijima pi extension can use **one** grant token (the principal's full
  capability set) instead of a 4–6-token env bundle.
- Dominic can hold one federation-grant token.
- Singleton grants are backward-compatible with today's one-cap tokens.
- No change to the verify hot path's correctness.

## Relationship to Roadmap #18 (Flag Variety Embedding)

The v0.4.0 implementation uses partition comparison within a single
Grassmannian. Roadmap #18 (Cross-Domain Flag Variety Embedding) generalizes
this to cross-domain grants via flag varieties:

| | v0.4.0 (single Grassmannian) | #18 (flag variety) |
|---|---|---|
| **Containment check** | Partition comparison: λ ≥ μ component-wise | Flag variety intersection theory |
| **Cost** | O(k) per capability | O(complex) — needs #18 research |
| **Admin imply** | Automatic (σ₄₄₄₄ = max partition) | Automatic (point class in flag) |
| **Cross-domain** | Not supported | Native — flags chain subspaces |

The API (`may(cap)`) does not change between v0.4.0 and #18 — only the
internal containment check upgrades. A grant on a single Grassmannian is a
union of Schubert varieties; a cross-domain grant (capabilities on both
Gr(k₁,n) and Gr(k₂,n)) lives naturally on the flag variety Fl(k₁, k₂, n).

## Implementation Checklist

- [ ] `GrantToken` type carrying `Vec<CapabilityId>` + signature
- [ ] `issue_grant()` on `CapabilityIssuer`
- [ ] `may(grant, cap)` using partition component-wise comparison
- [ ] `CapabilityIssuer::to_bytes()` / `from_bytes()` (item #16.1, shared)
- [ ] Rate-limit bucket using min-codimension
- [ ] Canonical sorting of grant for signing surface
- [ ] Backward compatibility: singleton grants equivalent to current tokens
- [ ] Tests: write-implies-read, admin-implies-all, rate-limit-keyed-on-min-codim
- [ ] Ijima integration test: one grant token replaces env-bundle

## References

- Ijima pi integration plan: `../Ijima/docs/plans/2026-07-12-pi-integration.md` (§5 token strategy, §11).
- Ijima capability vocabulary: `../Ijima/policy/policy.toml`, `ijima-core/src/capabilities.rs`.
- Current Schubert token model: `src/crypto.rs` (`CapabilityToken`, `issue`, `issue_batch`).
- Roadmap: `docs/ROADMAP.md` items #16.4 (this feature) and #18 (flag variety generalization).
