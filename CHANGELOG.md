# Changelog

## [0.2.0] — Unreleased

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
