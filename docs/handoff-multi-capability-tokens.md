# Handoff: Multi-Capability Tokens (requirements from Ijima + Dominic)

> **Status:** Requirements for the next Schubert release. Driven by Ijima's
> pi integration (PR #35 plan) and Dominic's federation orchestration.
> Author: Ijima session, 2026-07-12.

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

- A `Grant` (or multi-cap `CapabilityToken`) carries `capabilities: Vec<CapabilityId>`
  (or a set/bitset) instead of one.
- `verify(token, required_cap)` succeeds iff `required_cap` is in the grant.
- A singleton grant (`[cap]`) is exactly today's one-cap token — **backward-compatible**.

## Design alignment with Schubert's geometric model

This fits the Grassmannian naturally: a grant is a **subvariety** of Gr(4,8)
(the union/sum of the granted Schubert varieties), not a single partition.
`may(cap)` becomes *"does the granted subvariety contain `cap`'s Schubert
variety?"* — the existing intersection semantics, generalized from one cap
to a set. A grant's "size" (rate-limit relevance) is some aggregate of the
granted caps' codimensions.

## Considerations to resolve in the Schubert design pass

1. **Signing surface** — sign over `principal || cap_set || issuer_key`
   (cap_set canonically ordered/encoded).
2. **`may(cap)` semantics** — set membership vs. geometric containment;
   decide whether a grant implies *exactly* its caps or closure under
   intersection.
3. **Rate-limit bucket** — currently keyed on the single cap's codimension.
   For a multi-cap grant: key on the principal, or on an aggregate (e.g.
   min codimension, or sum)? Needs a deliberate choice (Ijima's rate-limiting
   wires Schubert codimensions → throughput).
4. **Wire format** — the bearer token encoding must carry the cap list
   compactly (bitset over the vocabulary? canonical id list?). Keep it
   URL/header-safe.
5. **Admin/imply** — does an `admin` grant (the point class σ₄₄₄₄) encoded
   as a multi-cap grant still imply everything? Preserve the "admin implies
   all" invariant.
6. **Issuance API** — `issue(principal, caps: IntoIterator<CapabilityId>)`
   returning one grant token; keep `issue_batch` for the N-separate-tokens
   case if still useful.

## Consumer acceptance (what unblocks Ijima/Dominic)

- Ijima pi extension can use **one** token (the principal's grant) instead
  of a 4–6-token env bundle.
- Dominic can hold one federation-grant token.
- No change to the verify hot path's correctness; one-cap tokens still work.

## References

- Ijima pi integration plan: `../Ijima/docs/plans/2026-07-12-pi-integration.md` (§5 token strategy, §11).
- Ijima capability vocabulary: `../Ijima/policy/policy.toml`, `ijima-core/src/capabilities.rs`.
- Current Schubert token model: `src/crypto.rs` (`CapabilityToken`, `issue`, `issue_batch`).
