# ADR-0001: GrantToken expiry semantics

**Status:** Proposed (2026-08-17) — scope decision for Roadmap #20.1 (v0.5.0).
**Consumers:** Ijima v0.2.0 (named expiry as its 0.5 feature request; WS1b revocation
is the interim kill-switch), Wallace extensions (session-scoped grants), Dominic
(time-boxed federation grants).
**Purpose:** define the expiry surface *before* Ijima builds against an undefined
0.5. Half a page, per PULSE 2026-08-17 rec #5.

## Decision (proposed)

1. **`expires_at` is a signed field.** Add optional `expires_at: Option<u64>` (Unix
   seconds) to `GrantToken`, covered by the Ed25519 signature — tamper-proof, no
   external state needed to check it. `None` = no expiry (today's behavior,
   backward compatible).
2. **`GrantVerifier::verify` rejects expired grants.** Verification order:
   signature → expiry → (caller-side) revocation. Expiry is checked against the
   verifier's wall clock at verify time; a `verify_at(instant)` variant supports
   deterministic tests and replay.
3. **Clock skew is the verifier's problem, not the token's.** No skew fields in v1;
   document that verifiers near trust boundaries may apply their own tolerance.
   Do not over-engineer.
4. **Renewal = re-issue.** Tokens are immutable and signature-covered; "extending" a
   grant means issuing a new token (new content hash → revocation sets treat it as a
   distinct token). No renewal protocol in 0.5.
5. **Revocation composes as OR.** Access = signature-valid AND not-expired AND
   not-revoked (Ijima WS1b store-backed set). Expired tokens *may* remain in
   revocation sets harmlessly (different hash namespaces; garbage-collect on
   operator schedule).

## Wire-format impact (the one breaking consideration)

`to_bytes`/`from_bytes` gain an optional expiry field. Options: (a) bump the format
version marker, rejecting old encodings; (b) append-only optional field with a
presence flag, accepting old blobs. **Recommend (b)** — grants are young, but Ijima
0.1.0 CapabilityToken-era blobs already exist in the wild and tolerant reads cost
nothing. tsukoshi's `GrantCRDT`/crypto subpath mirrors whichever is chosen
(Rust-compatible wire format is its contract).

## Consequences

- Ijima can implement "session-scoped + hygiene expiry" with one field and a wall
  clock; Wallace extension manifests can request `expires_at` aligned to session
  lifetime.
- No renewal/rotation protocol — deliberate; revisit if operational pain appears.
- Schubert 0.5 scope stays small: this ADR + grant-aware CRDT revocation (#20.2) +
  policy→issuance linkage (#20.3).
