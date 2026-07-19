# Changelog

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
