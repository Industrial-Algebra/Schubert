# ADR-0001: GrantToken expiry semantics

**Status:** Proposed (2026-08-17; revised same day — reconciled with the fuller
#20 spec sketched by the Schubert session, which supersedes this ADR's simpler
renewal model). Scope: Roadmap #20.1 (v0.5.0).
**Consumers:** Ijima v0.2.0 (named expiry as its 0.5 feature request; WS1b revocation
is the interim kill-switch), Wallace extensions (session-scoped grants), Dominic
(time-boxed federation grants).
**Purpose:** define the expiry surface *before* Ijima builds against an undefined
0.5. Half a page, per PULSE 2026-08-17 rec #5.

## Decision (proposed)

0. **`nonce: [u8; 16]` — issuance distinctness.** Random at issue, covered by
   the signature, *not* part of the canonical capability sort. Motivation
   (from the #20 spec, Ijima-driven): `issue_grant` currently signs only
   `(principal, capabilities, issuer_key)`, so re-issuing the same grant from
   the same seed produces a byte-identical bearer — a revoked token cannot be
   cleanly re-issued. The nonce makes every issuance distinct.

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
4. **Renewal = re-issue, enabled by a nonce.** Tokens are immutable and
   signature-covered; "extending" a grant means issuing a new token. This
   *requires* an issuance nonce (see rule 0): without one, deterministic
   issuance from the same seed yields a **byte-identical bearer**, and a
   revocation entry keyed by content hash would kill the re-issue along with
   the original. No renewal protocol in 0.5 — document the pattern.
5. **Revocation composes as OR.** Access = signature-valid AND not-expired AND
   not-revoked (Ijima WS1b store-backed set). Expired tokens *may* remain in
   revocation sets harmlessly (different hash namespaces; garbage-collect on
   operator schedule).

## Wire-format impact (the one breaking consideration)

`to_bytes`/`from_bytes` gain `expires_at` + `nonce`. **Revised stance (consumer-
informed):** a breaking blob-layout change is **acceptable in a 0.x minor** —
Ijima is the sole bearer-holder and re-mints on upgrade; tsukoshi mirrors in
lockstep via the cross-language fixture tests (Rust-issued ⇒ TS-verified, both
expiry and nonce paths). Tolerant append-only reads remain an option if
downgrade compatibility ever matters; don't pay for it now.

## Consequences

- Ijima can implement "session-scoped + hygiene expiry" with one field and a wall
  clock; Wallace extension manifests can request `expires_at` aligned to session
  lifetime.
- No renewal/rotation protocol — deliberate; revisit if operational pain appears.
- Schubert 0.5 scope stays small: this ADR + grant-aware CRDT revocation (#20.2) +
  policy→issuance linkage (#20.3).

## References

- Roadmap #20 (this repo) — the full Ijima-driven expiry & nonce spec.
- Ijima `docs/adr/token-revocation.md` — revocation = incidents; expiry =
  routine deprovisioning (complementary rationale).
- Ijima `docs/adr/grant-token-migration.md` — consumer context.
- `PULSE_2026-08-17_Anima.md` rec #5 — the scoping request this answers.
