# Schubert — Directions

> **v0.1.0 Snapshot** — All 14 core roadmap items are complete. The speculative
> directions below are explorations for the research community, not commitments.
> See [CHANGELOG.md](../CHANGELOG.md) for the full v0.1.0 feature list.

**Version:** 0.1.0 — Foundation complete. IA-conformant. Licensed.
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
- ✅ AGPL-3.0 dual-licensing

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
AGPL-3.0 licensed). Capabilities are binding vectors in a holographic
reduced representation. Access is granted when the query vector's
similarity to the capability vector exceeds the trust threshold.
The wall-crossing engine determines which memories are accessible at
each trust level.

---

## Design Principles (Preserved Across All Directions)

1. **No network service.** Schubert remains a library. Deployment patterns (daemon, sidecar, plugin) are the caller's choice.

2. **Identity is external.** Schubert never authenticates. It authorizes against identities provided by the caller.

3. **Mathematics is the authority.** Every access decision has a geometric basis. No heuristic rules, no pattern matching, no ML models.

4. **Synchronous by default.** The core API is synchronous. Async wrappers can be built externally.

5. **Embeddable.** A single dependency. Compiles anywhere Rust compiles. Suitable for kernels, embedded systems, browsers.

---

*Schubert 0.1.0 — May 2026*
