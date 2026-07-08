# Schubert — Directions

> **v0.3.0 Snapshot** — All 14 core roadmap items complete. Karpal/Minuet
> upgraded to Apache-2.0. Formal mapping substantiated via distributed game
> sync design. 14 Proserpina critique findings addressed.
> See [CHANGELOG.md](../CHANGELOG.md) for version history.

**Version:** 0.3.0 — Clean AGPL break. Formal foundation. Apache-2.0 throughout.
**Gitflow:** `main` (releases) ← `develop` (integration) ← `feature/*` (work)

---

## Current State

Schubert provides a practical access control library built on Schubert calculus.
It is embeddable, synchronous, and depends on `amari-enumerative` v0.22 (plus
optional `karpal-proof`, `karpal-verify`, `serde`, `rayon`, `toml`).
85 unit tests, 0 warnings across all feature combinations.

**Completed since foundation:**
- ✅ IA ecosystem conformance, serde, karpal, parallel, policy, wasm, crypto
- ✅ Computation path selection (4 paths + auto-routing)
- ✅ Serialization roundtrip (AccessController serde + JSON/file I/O)
- ✅ Policy language (declarative TOML)
- ✅ WebAssembly target (wasm32-unknown-unknown)
- ✅ Context-aware decisions (resource scoping + time-aware trust)
- ✅ Multi-Grassmannian controllers (cross-domain access)
- ✅ Proof-carrying capabilities (Ed25519 cryptographic tokens)
- ✅ Constitutional verification (Karpal 0.5.0 integration)
- ✅ Apache-2.0 dual-licensing

---

## Near-Term (Practical)

### 1. Computation Path Selection — ✅ DONE (v0.1.0)

**Implemented:** All 4 amari computation paths exposed via `check_with_path()`:
- `LittlewoodRichardson` — exact, classical (default)
- `Localization` — equivariant localization (Atiyah-Bott), better scaling for large Gr(k,n)
- `Tropical` — tropical intersection (fast, approximate counts)
- `Matroid` — polynomial-time independence check

Auto-routing via `check_auto()`: Gr(k,n) with n ≤ 8 uses LR, larger uses Localization.

**Verified:** σ₁⁴=2 and σ₂·σ₁₁=0 consistent across LR, Localization, and Tropical paths.
Matroid correctly detects impossibility.

### 2. Serialization and Persistence — ✅ DONE (v0.1.0)

**Implemented:** Full `AccessController` serde with roundtrip fidelity:
- `Serialize` + `Deserialize` on `AccessController` (audit sink skipped)
- `Principal.granted_capability_ids` tracks grants for namespace reconstruction
- `rebuild_principal_namespaces()` restores amari namespaces after deserialization
- `to_json()` / `from_json()` convenience methods
- `save_to_file()` / `load_from_file()` file I/O (requires `std`)

**Verified:** 8 roundtrip tests covering empty controller, capabilities, principals,
access decisions (including σ₁⁴=2 and σ₂·σ₁₁ impossibility), grants, revokes, and file I/O.

### 3. Policy Language — ✅ DONE (v0.1.0)

**Implemented:** Declarative TOML policy format with full validation:
- `PolicyConfig` struct with serde Deserialize/Serialize
- `from_policy_toml()` / `to_policy_toml()` on `AccessController`
- Grassmannian validation, partition bounds checks, weakly-decreasing check
- Principal grant reference validation
- `examples/policies/rbac.toml` — complete Kubernetes RBAC policy file

**Verified:** 15 policy tests (parse, validate, apply, roundtrip, error cases, file loading).

### 4. WebAssembly Target — ✅ DONE (v0.1.0)

**Implemented:** Full wasm32-unknown-unknown compatibility:
- `wasm` feature propagates to `amari-enumerative/wasm`
- `WasmController` — wasm-bindgen wrapper with full JavaScript API
- `AuditSink` gated behind `std` feature (not available on wasm)
- `InMemoryAudit` uses `RefCell` in no_std, plain `Vec` on wasm
- `now_millis()` returns 0 on wasm32
- CI checks: wasm32 build without features + with `wasm` feature

**Verified:** Compiles cleanly for `wasm32-unknown-unknown` with both
`--no-default-features` and `--features wasm`.

### 5. Context-Aware Decisions — ✅ DONE (v0.1.0)

**Implemented:** `AccessContext` with resource, time, and metadata:
- `check_with_context()` extends standard checks with resource scoping
  and time-based trust degradation
- Resource-scoped capabilities: `"cap/resource_id"` checked in addition
  to base capability when context.resource is set
- Time-aware trust: trust factor decays linearly from 1.0 (fresh) to
  0.0 (2+ years old), scaling configuration counts
- Builder methods: `AccessContext::empty()`, `for_resource()`, `at_time()`

**Verified:** 4 tests (resource scoping, empty context matching standard
check, time degradation, no-time no-degradation).

---

## Medium-Term (Research-Adjacent)

### 6. Multi-Grassmannian Controllers — ✅ DONE (v0.1.0)

**Implemented:** `MultiController` managing multiple Grassmannian domains:
- `add_domain(k, n)` / `add_domain_named(k, n, label)` — register domains
- `create_principal()`, `grant_in_domain()`, `register_in_domain()` — per-domain ops
- `check_in_domain()` — standard check within a domain
- `check_cross_domain()` — translate capabilities between Grassmannians
  via partition validation (fits-in-box check)
- `translatable_capabilities()` — list capabilities compatible across domains
- `domains_for_partition()` — find domains that accept a given partition

**Verified:** 7 tests (add domains, same-domain check, cross-domain translatable,
cross-domain check, denied-if-not-held, partition-based domain discovery,
duplicate label rejection).

### 7. Proof-Carrying Capabilities — ✅ DONE (v0.1.0)

**Implemented:** Full cryptographic capability tokens via Ed25519:
- `CapabilityToken` — signed assertion of principal+capability
- `CapabilityIssuer` — generates key pairs and issues signed tokens
- `CapabilityVerifier` — verifies signatures and extracts claims
- `verify_batch()` for parallel verification (behind `parallel` feature)
- Tamper-detection: modified tokens fail signature verification

**Verified:** 6 tests (issue+verify, wrong key, tampered capability,
tampered principal, verify_and_extract, batch issuance).

### 8. Temporal Access Control — ✅ DONE (v0.1.0)

**Implemented:** Timed capabilities with automatic expiry:
- `Capability::expires_at` — optional Unix timestamp expiry
- `Capability::with_expiry()` — builder pattern for timed capabilities
- `Capability::is_expired_at()` / `time_remaining_at()` — expiry queries
- `AccessController::check_temporal()` — access checks with expiry awareness
- `expired_capabilities()` / `capability_time_remaining()` — temporal queries
- `temporal_trust_level()` — linear trust decay from grant to expiry

**Verified:** 6 tests (expired denied, no-expiry always allowed, mixed expiry,
expired listing, trust decay, time remaining).

### 9. Quantitative Rate Limiting — ✅ DONE (v0.1.0)

**Implemented:** Token-bucket rate limiting scaled by Schubert intersection numbers:
- `RateLimiter` — per-principal token buckets with capacity = intersection_number × multiplier × base_rate
- `configure_principal()` / `configure_from_decision()` — setup from access results
- `try_consume()` / `can_consume()` — token consumption with refill
- `tokens_available()` / `capacity()` — bucket state queries
- Linear refill: tokens replenish at configured rate over time

**Verified:** 7 tests (consume, exhaust, higher-intersection-gets-more,
configure from Granted/Denied, can_consume, remove principal).

---

## Far-Term (Speculative)

### 10. Schubert Routing — ✅ DONE (v0.1.0)

**Implemented:** Geometric network routing via Schubert calculus:
- `RouteTable` — manages route advertisements as Schubert conditions
- `RouteAdvertisement` — partition + hop count per node
- `check_route()` / `check_path()` — single-hop and multi-hop path computation
- `congestion_level()` — codimension/dimension ratio for congestion detection
- Intersection number = valid route count

**Verified:** 7 tests (single hop, congested, impossible, multi-hop,
congestion level, missing node, node listing).

### 11. Surreal Trust Levels — ✅ DONE (v0.1.0)

**Implemented:** Full surreal trust via Amari 0.23.0:
- `RationalSurreal` — exact rational trust (1/2, 3/7, etc.)
- `EpsilonPolynomial` — infinitesimal trust (ε, ε², 5ε)
- `SurrealTrust::epsilon()` / `epsilon_power(n)` — infinitesimal hierarchy
- Exact ordering: ε > ε² > 0, 0.5+ε > 0.5
- `from_f64()` / `approximate()` for backward compatibility
- `has_infinitesimal()` / `is_purely_finite()` detection

**Verified:** 8 tests (full/none, rational ordering, epsilon positive,
epsilon hierarchy, mixed trust, roundtrip, conversion, detection).

### 12. Constitutional Verification — ✅ IMPLEMENTED (v0.1.0)

Integrated with Karpal 0.5.0 verification infrastructure:
- `verify::schubert_bundle()` — 5 proof obligations
- `verify::certify_capability()` — certified trust boundary
- SMT-LIB2 and Lean 4 export support
- CI: `.github/workflows/schubert-verify.yml`

### 13. Distributed Access Control with CRDTs — ✅ DONE (v0.1.0)

**Implemented:** Eventually-consistent capability grants via CRDTs:
- `CrdtState` — mergeable access control state with version vectors
- `VersionVector` — happens-before comparison, merge (pointwise max)
- `CrdtGrant` — last-write-wins grant with version tracking
- `merge()` — commutative, associative, idempotent state merge
- `check()` — access computation from eventually-consistent state

**Verified:** 8 tests (happens-before, concurrent, grant/hold, revoke,
merge-preserves, last-write-wins, idempotent, crdt-access-check).

### 14. Access Control for Holographic Memory — ✅ DONE (v0.1.0)

Integration with Minuet-style holographic memory systems (Minuet 0.3.0,
Apache-2.0 licensed). Capabilities are binding vectors in a holographic
reduced representation. Access is granted when the query vector's
similarity to the capability vector exceeds the trust threshold.
The wall-crossing engine determines which memories are accessible at
each trust level.

---

## Near-Term (v0.4.0)

### 15. schubert-tsukoshi — Pure TypeScript Access Control

**Goal:** Extract Schubert's core access control model into a zero-dependency
TypeScript package, following the cliffy-tsukoshi pattern.

**Architecture:**
- **LR coefficient tables** — precomputed lookup tables for Gr(2,4), Gr(3,6),
  Gr(4,8). No amari-enumerative, no Rust, no WASM. Pure TS table lookups are
  faster than WASM for these small Grassmannians.
- **AccessController** — capability registry, principals, grant/revoke, check()
- **Impossibility detection** — the killer feature, via LR table returning 0
- **CRDT grants** — wraps cliffy-tsukoshi's VectorClock + GeometricCRDT for
  distributed access state
- **Universal deployment** — browser, Node.js, React Native, Deno

**What it provides:**
- Browser-native geometric access control (no backend required)
- Impossibility detection in JavaScript — the σ₂·σ₁₁ = 0 case works in the browser
- CRDT-backed distributed access (leveraging cliffy-tsukoshi's protocols)
- "Smuggle the mathematics" — JS developers call check(), never need to know
  what a Grassmannian is

**What it doesn't need:**
- No karpal verification (requires Rust type system)
- No surreal trust (use `number` for trust in JS)
- No minuet holographic (no holographic memory in browser)

**Scope:** ~1 week of focused work. Published as `@ia/schubert-tsukoshi` on npm.

**Reference:** [cliffy-tsukoshi](https://github.com/justinelliottcobb/Cliffy/tree/main/cliffy-tsukoshi) —
the pattern this follows (pure TS extraction of geometric math from a Rust
framework, with distributed protocols).

### 16. Consumer-Driven API Polish (from Ijima Integration)

**Motivation:** Ijima — Schubert's first real consumer — revealed integration
friction points. Each item below eliminates custom boilerplate Ijima had to
write.

**1. `CapabilityToken::to_bytes()` / `from_bytes()`**
Ijima reimplemented 80 lines of custom binary wire format (length-prefixed
fields + base64). Add native serialization to eliminate per-consumer wire
format code.

**2. `CapabilityIssuer::from_seed()` + `public_key_hex()`**
Ijima wraps `ed25519_dalek::SigningKey::from_bytes()` and manually hex-encodes
the public key. Add convenience methods directly on the issuer.

**3. `schubert::axum` module** (feature-gated)
Ijima wrote its own `AuthPrincipal` Axum extractor (100 lines). Provide built-in
extractors and middleware so consumers don't reinvent the integration layer.

**4. Multi-capability tokens**
Tokens currently carry one capability. Ijima's `require()` checks
`token.capability == required || token.capability == ADMIN`. Support tokens
that carry `Vec<CapabilityId>` to reduce token management overhead.

**5. Key persistence utilities**
Ijima wrote 180 lines of file-based key storage (`key_store.rs`) with `0600`
permissions, path resolution, and load-or-create semantics. Provide a `KeyStore`
utility or document the recommended pattern.

**6. `check_single()` fast path**
Ijima bypasses the geometric intersection for per-request checks (uses string
comparison) because the full `check()` is too heavy for runtime. Provide a
lightweight single-capability check suitable for high-throughput request paths.

**Scope:** ~2–3 days of focused work. All items are directly validated by
real consumer usage.

---

## Research Directions (v0.5.0+ and Beyond)

### 17. Compositional Wall-Crossing

**Origin:** The stability-engine rabbit hole (2026-07-06) identified this as
Schubert's deepest open theoretical question.

**The question:** Does the wall-crossing phase diagram compose under operadic
gluing? If Principal C = A ∘_S B (composed along shared capability set S), is
the phase diagram P_C determined by P_A and P_B?

**Why it matters:** In physics, BPS bound states have different wall-crossing
behavior than their constituents. The Kontsevich-Soibelman (KS) formula
governs how the spectrum changes when crossing a wall. If Schubert's
composition satisfies a KS-type formula, then wall-crossing composes — and
Schubert is not just a geometric access control system but a **category**: 
principals as objects, compositions as morphisms, wall-crossing as a natural
transformation from trust levels to stable capability sets.

**Implementation:**
- Add `analyze_composed_stability()` that takes two principals + shared
  capabilities and returns the composed phase diagram
- Compare against individual phase diagrams to test for a KS-type relation
- If confirmed, this becomes the arXiv preprint's central theoretical result

**Scope:** Research-grade. Requires formal mathematical work alongside
implementation. Directly informs the publication strategy (Ch. 8 of the
revised preprint outline).

### 18. Cross-Domain Flag Variety Embedding

**Origin:** Deferred Proserpina critique finding — cross-domain intersection
derivation needs a flag variety embedding proof.

**The question:** Can capability translation between Grassmannians
Gr(k₁,n₁) → Gr(k₂,n₂) be formalized as an embedding into a common flag
variety Fl(k₁, k₂, n)?

**Why it matters:** The `MultiController` currently translates capabilities
between Grassmannians heuristically. A flag variety embedding would provide
the formal mathematical foundation — and would close the gap between the
multi-domain implementation and its theoretical justification.

**Scope:** Paper material. Requires algebraic geometry expertise. Maps to
Ch. 12 (Future Work) of the arXiv preprint.

### 19. GPU-Accelerated Schubert Calculus

**Origin:** Borsalino (the IA GPU abstraction layer) provides WGSL kernel
dispatch for geometric algebra operations.

**The opportunity:** Large Grassmannians (Gr(k,n) with n > 8) require
computation paths beyond Littlewood-Richardson. Borsalino's GPU substrate
could accelerate equivariant localization and tropical intersection
computations, making large-Grassmannian access control practical.

**Scope:** Depends on Borsalino reaching ecosystem adoption. Long-term.

---

## Design Principles (Preserved Across All Directions)

1. **No network service.** Schubert remains a library. Deployment patterns (daemon, sidecar, plugin) are the caller's choice.

2. **Identity is external.** Schubert never authenticates. It authorizes against identities provided by the caller.

3. **Mathematics is the authority.** Every access decision has a geometric basis. No heuristic rules, no pattern matching, no ML models.

4. **Synchronous by default.** The core API is synchronous. Async wrappers can be built externally.

5. **Embeddable.** A single dependency. Compiles anywhere Rust compiles. Suitable for kernels, embedded systems, browsers.

---

*Schubert 0.1.0 — May 2026*
