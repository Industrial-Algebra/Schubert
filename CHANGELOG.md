# Changelog

## [0.5.0] — Unreleased

### Added

- **`GrantToken` expiry & nonce (#20.1, ADR-0001)** — token-carried grant
  lifecycle, verifier-checked standalone (no controller round-trip — what
  federation satellites need):
  - `expires_at: Option<u64>` (Unix seconds, **covered by the signature**;
    `None` = never, the pre-0.5.0 behavior). A grant is dead the instant
    `now >= expires_at` (inclusive boundary).
  - `nonce: [u8; 16]` — random at issue, signed, **not** part of the canonical
    capability sort. Makes every issuance distinct, so a revoked grant can be
    cleanly re-issued from the same seed (renewal = re-issue, ADR-0001 rule 4).
  - `GrantVerifier::verify_at(grant, now_unix)` — deterministic, clock-injected;
    `verify(grant)` delegates via `SystemTime::now()`. Verification order:
    signature, then expiry. New [`SchubertError::GrantExpired`] distinguishes a
    dead grant from a forged one.
  - Issuer surface: `issue_grant` (defaults), `issue_grant_with_expiry`,
    `issue_grant_with_options(GrantOptions)` with injectable nonce for
    deterministic tests.
  - **Breaking wire-format change** (ADR-0001: acceptable in 0.x — Ijima, the
    sole bearer holder, re-mints): `to_bytes`/`from_bytes` gain trailing
    `nonce(16) | tag(1) | [expires_at u64 BE]`. tsukoshi parity follows.

- **Grant-aware CRDT revocation (#20.2, ADR-0002)** — tombstone registry for
  *specific issuances*: a grow-only set keyed by the #20.1 nonce, merged by
  **union**, so a tombstoned grant can never be resurrected by any merge order
  (the add-wins LWW map could not guarantee that). Renewal = re-issue stays
  clean — the rotated grant carries a fresh nonce.
  - Rust: `CrdtState::revoke_grant(nonce, node, ts)` / `is_grant_revoked(nonce)`;
    blanket `(principal, capability)` `revoke` unchanged.
  - tsukoshi: `GrantCRDT.revokeGrant(nonceHex)` / `isGrantRevoked(nonceHex)`;
    snapshots carry the set, and pre-v0.5.0 snapshots deserialize to an empty
    set (backward-tolerant).
  - Access predicate now: valid signature AND not expired AND **not
    tombstoned** AND not blanket-revoked (ADR-0001 rule 5, extended).

- **Policy → issuance linkage (#20.3)** — policy.toml now constrains *what
  grants may be issued*, so consumers (Ijima) can own policy while principals
  carry proof-carrying grants:
  - `GrantPolicy::from_policy(&PolicyConfig)` — validates, then derives the
    entitlement map (gated `crypto` + `policy`).
  - `GrantPolicy::may_issue(principal, caps)` — `Ok(())` iff every requested
    `(id, partition)` pair **exactly** matches the entitlement; fails closed
    (unknown principal = deny; partition mismatch = deny — no smuggling a
    stronger geometry under an allowed id).
  - `issue_grant_under_policy(issuer, policy, principal, caps, options)` —
    check-then-sign in one seam, honoring [`GrantOptions`] (nonce/expiry).
  - New [`SchubertError::GrantDeniedByPolicy { principal, capability }`].
  - Feature-free view: `PolicyConfig::grants_for(principal)` returns the
    entitled `(id, partition)` pairs.

- **#17 instrument** — `analyze_composed_stability()`: computes `P_A`/`P_B`/`P_C`
  and tests the KS-type additivity question empirically, plus the
  `wall_crossing_probe` example sweeping a family of compositions. **Fix found
  by the probe:** overlapping retained capabilities are deduplicated in the
  additive baseline (union semantics — previously every overlap read as false
  emergence); regression-tested.

### Documentation

- Rewrote AGENTS.md test baselines (219 all-features: 183 lib + 18 CLI + 18
  doc; TypeScript 53: 12 controller + 18 crypto + 23 CRDT) and the
  `dist/`-is-gitignored publish note.
- New book sections across the sprint: `crypto.md` *Grant Expiry & Nonce*,
  `axum.md` *expiry ⇒ 401*, `crdt.md` *Grant Tombstones*, `policy.md`
  *Constrained Issuance*, `feature-flags.md` `crypto`×`policy` combination,
  `tsukoshi.md` v0.5.0 subpath capabilities + two-layer composition note.
- README v0.5.0 *What's New*; tsukoshi README documents the lifecycle options,
  `verifyGrantAt`, and tombstone incident-response flow.


## [0.4.0] — 2026-07-19

### Added

- **`schubert::axum` module** — `AuthPrincipal`, a bearer-token extractor that
  validates a `GrantToken` from the `Authorization: Bearer <token>` header. Typed
  rejections split client vs server faults (`AuthError::Unauthorized` → 401,
  `AuthError::ServerMisconfigured` → 500) with a uniform 401 body that leaks no
  validation-stage detail. Enabled by the new `axum` feature (pulls `crypto`).
- **Multi-capability grant tokens** — `crypto::GrantToken`, `GrantCapability`,
  and `GrantVerifier`. A grant carries several capabilities, each with its
  Schubert partition, so a verifier answers *does this grant authorize P?* via
  `GrantVerifier::may(grant, partition)` using geometric containment — write
  implies read, admin (the max partition) implies all — with **no capability
  registry** at verification time. Capabilities are canonically sorted before
  signing, so grant order does not affect the signature.
- **`crypto::KeyStore`** — file-based Ed25519 seed persistence with load-or-create
  semantics (`load_or_create` / `load` / `generate_seed`). Atomic against
  concurrent startup; files created mode `0600` on Unix.
- **Binary wire format** — `CapabilityToken::to_bytes()` / `from_bytes()` and
  `GrantToken::to_bytes()` / `from_bytes()`, a length-prefixed format suitable
  for base64 bearer tokens.
- **Issuer convenience** — `CapabilityIssuer::from_seed([u8; 32])` (deterministic
  restoration) and `public_key_hex()` (64-char lowercase hex for display/config).
- **`AccessController::check_single()`** — a lightweight set-membership fast path
  for per-request checks that bypasses geometric intersection.
- **`schubert-tsukoshi`** (separate npm package
  `@industrialalgebra/schubert-tsukoshi`) — a pure-TypeScript extraction of the
  access-control model: zero-dependency core (LR tables for Gr(2,4)/Gr(3,6)/
  Gr(4,8), impossibility detection), an Ed25519 `crypto` subpath with a
  **Rust-compatible wire format** (tokens interop both directions), and a
  `protocols` subpath with `GrantCRDT` (replicated grant set over cliffy-tsukoshi's
  `VectorClock`).
- **`axum` feature flag** (optional deps `axum`, `base64`; enables `crypto`).

### Documentation

- **Rewrote `api/crypto.md`** — the prior page documented a non-existent API
  (`serialize()`, wrong `verify_and_extract`, `CapabilityToken` fields that
  don't exist). Now covers the real surface: grant tokens, `may()` containment,
  the wire format, and `KeyStore`.
- **New `api/axum.md`** — the extractor, the 401-vs-500 split, capability-specific
  authorization via `may()`.
- **New `api/tsukoshi.md`** — cross-reference to the TypeScript package.
- **`guide/feature-flags.md`** — added the `axum` feature; expanded the `crypto`
  description; added a web-service feature combination.

### Changed

- Bumped to `0.4.0` (minor): backward-compatible additions only.


## [0.3.0] — 2026-07-04

### Changed
- **Karpal upgraded 0.5 → 0.6.1** (Apache-2.0, new `compose_checks()` API)
- **Minuet upgraded 0.3 → 0.5.0** (Apache-2.0, ShardedStore support)

### Added
- **Distributed game sync design** (`docs/design/distributed-game-sync.md`) —
  formal mapping from game state to Grassmannian, operational definition of
  "configuration," concrete impossibility detection case (σ₂·σ₁₁ = 0).
  Addresses the foundational critique blockers.
- **Threat model** (`book/design/threat-model.md`) — adversary capabilities,
  security properties, audit trail discussion, non-identifiability clarification.
- **Flag structure** (`book/design/flag-structure.md`) — reference flag as
  clearance hierarchy, explains why σ₂ and σ₁₁ differ despite same codimension.
- **Adversarial concerns** (`book/design/adversarial-concerns.md`) — DoS
  mitigation, dimensionality poisoning, CRDT state poisoning, timing channels.
- **`karpal_compose` module** (`src/proof.rs`) — type-level triple-composition
  verification via `SchubertProven::compose_checks()`.
- **`HolographicStore` enum** (`src/holographic.rs`) — Simple + Sharded store
  variants with `new_sharded()` constructor for production-scale holographic storage.
- **Notation guide** in concepts/math.md (σ symbols, point class defined).
- **Prior art references** in concepts/math.md.
- **Rust quickstart code** in book introduction.
- **Dependency table** clarifying optional vs required crates.

### Documentation Improvements
- Relabeled "verified mathematical properties" → "algebraic identities" with
  caveat that security relevance depends on the domain mapping.
- Moved "What's New" section from math foundation to end of README.
- 14 of 22 Proserpina critique findings addressed.

## [0.2.0] — 2026-06-30

### Changed
- **License**: Relicensed from AGPL-3.0-only to **Apache-2.0 with CLA**.
  Removes the network-use clause that blocked enterprise adoption.

### Added
- **Benchmarks**: `criterion` benchmarks comparing all 4 computation paths
  (Littlewood-Richardson, Localization, Tropical, Matroid) on Gr(2,4),
  Gr(3,6), and Gr(4,8).
- **Deployment example**: Axum web middleware (`examples/deployment/axum_middleware.rs`)
  demonstrating Schubert as an HTTP authorization layer.
- **CRDT staleness gating**: `CrdtState::set_max_staleness()`, `staleness_ms()`,
  and `is_converged_with()` for guarding access decisions on
  eventually-consistent state.
- **Architectural philosophy**: New book section documenting the
  "exact math, approximate infrastructure" design boundary.

### Removed
- **LICENSE-COMMERCIAL**: No longer needed under Apache-2.0.
- **Dual-licensing references**: All docs updated to Apache-2.0 + CLA.

## [0.1.0] — 2026-06-05

### Added
- **Core**: `AccessController` with principal management, capability registry, grant/revoke
- **Decisions**: Quantitative `AccessDecision` (Granted{n}, Impossible, Denied, Underconstrained)
- **Computation paths**: 4 engines (LR, Localization, Tropical, Matroid) with auto-routing
- **Composition**: Operadic composition via `compose()` and `are_composable()`
- **Stability**: Wall-crossing engine with `analyze_stability()` and phase diagrams
- **Audit**: Pluggable `AuditSink` trait with `InMemoryAudit` implementation
- **Serialization**: `serde` feature with JSON I/O, `AccessController` roundtrip
- **Policy language**: TOML format via `policy` feature, `from_policy_toml()`/`to_policy_toml()`
- **WebAssembly**: `wasm` feature with `WasmController` JS bindings
- **Context-aware**: `AccessContext` with resource scoping and time-aware trust
- **Multi-Grassmannian**: `MultiController` with cross-domain capability translation
- **Proof-carrying**: Karpal `proof` module (Proven, Property hierarchy, Rewrite, law checks)
- **Cryptographic tokens**: Ed25519 `CapabilityToken` via `crypto` feature
- **Temporal access**: Timed capabilities with `expires_at`, `check_temporal()`
- **Rate limiting**: Token-bucket `RateLimiter` scaled by intersection numbers
- **Schubert routing**: `RouteTable` with geometric path computation
- **Surreal trust**: `SurrealTrust` via Amari 0.23 `RationalSurreal` + `EpsilonPolynomial`
- **Verification**: Karpal 0.5.0 integration with proof obligations and certified trust boundary
- **Distributed CRDTs**: `CrdtState` with version vectors and eventually-consistent merge
- **Holographic memory**: Minuet 0.3.0 `HolographicAccessControl` integration
- **Phantom types**: Re-exports of `amari_enumerative::phantom` for compile-time verification
- **CI/CD**: GitHub Actions for fmt, clippy, test (5 combos), doc, wasm, verification

### Infrastructure
- IA ecosystem conformance (rust-toolchain, phantom types, feature gates)
- 105 unit tests, 12 doc tests, 7 examples, 0 clippy warnings
- 18 source modules across 9 optional feature gates
- `docs/ROADMAP.md` with 14 speculative directions
- `docs/surreal-trust-levels.md` deep expansion
- `docs/verification-integration.md` design document
- `docs/guide/` — user guide, concepts, architecture, cookbook, feature flags

### Dependencies
- `amari-enumerative` v0.23 (Schubert calculus engine)
- `thiserror` v2 (error derive)
- Optional: `serde`, `serde_json`, `karpal-proof` v0.5, `karpal-verify` v0.5,
  `karpal-schubert-types` v0.5, `rayon`, `toml`, `wasm-bindgen`, `js-sys`,
  `ed25519-dalek`, `rand`, `amari-surreal` v0.23, `minuet` v0.3, `num-traits`, `num-bigint`
