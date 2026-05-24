# Schubert — Directions

**Version:** 0.1.0 — Foundation complete. IA-conformant. Licensed.
**Gitflow:** `main` (releases) ← `develop` (integration) ← `feature/*` (work)

---

## Current State

Schubert provides a practical access control library built on Schubert calculus.
It is embeddable, synchronous, and depends only on `amari-enumerative` (plus
optional `karpal-proof`, `serde`, `rayon`). The core API is implemented and
tested at 30 unit tests, 0 warnings.

**Completed since foundation:**
- ✅ IA ecosystem conformance (rust-toolchain, phantom types, no_std scaffolding)
- ✅ `serde` feature gate (derives on 11 key types)
- ✅ `karpal` integration (Proven, Property hierarchy, Rewrite, law checks)
- ✅ `parallel` feature gate (check_batch, stability_batch, compose_batch via rayon)
- ✅ AGPL-3.0 dual-licensing (commercial licenses available)

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

### 8. Temporal Access Control

**Current:** Capabilities are permanent until revoked.

**Direction:** Timed capabilities with automatic expiry. The stability engine naturally models this — a capability with expiry time T has a trust level that decays to zero at T, crossing walls at predictable intervals. The phase diagram becomes a calendar.

### 9. Quantitative Rate Limiting

**Current:** Access is binary-geometric. No notion of rate.

**Direction:** Blend Schubert intersection with token-bucket rate limiting. The intersection number determines the bucket capacity — access with 2 configurations gets 2× the rate of access with 1. The geometry of access maps to the geometry of throughput.

---

## Far-Term (Speculative)

### 10. Schubert Routing

Access decisions as routing rules. A network where route advertisement = capability grants and forwarding = Schubert intersection. The number of valid routes between source and destination is the intersection number. Congestion is codimension excess. This is the networking model explored in the ShaperOS transport layer, extracted as a standalone protocol.

### 11. Access Control for Holographic Memory

Integration with Minuet-style holographic memory systems. Capabilities are binding vectors in a holographic reduced representation. Access is granted when the query vector's similarity to the capability vector exceeds the trust threshold. The wall-crossing engine determines which memories are accessible at each trust level.

### 12. Surreal Trust Levels

When `amari-surreal` supports generalized surreal numbers beyond the dyadic layer, trust levels could be surreal-valued — enabling infinite descending chains of trust degradation. A capability that becomes unstable at trust level ε (infinitesimal) remains stable at all finite trust levels but can be distinguished from one that becomes unstable at ε².

### 13. Constitutional Verification — ✅ IMPLEMENTED (v0.1.0)

Integrated with Karpal 0.5.0 verification infrastructure:
- `verify::schubert_bundle()` — 5 proof obligations (LR consistency, partition
  validity, intersection emptiness, access idempotency, grant-revoke identity)
- `verify::verify_schubert()` — generates `VerificationReport` with
  `ProofTestCertificate` for each obligation
- `verify::certify_capability()` — wraps runtime validation in `Certified`
  trust boundary backed by proof-test evidence
- SMT-LIB2 and Lean 4 export via `export_schubert_smt()`/
  `export_schubert_lean()`
- CI: `.github/workflows/schubert-verify.yml` — proptest + SMT jobs
- 6 new tests verifying the verification infrastructure itself

### 14. Distributed Access Control with CRDTs

Operadic composition over a distributed system using conflict-free replicated data types. Principals hold vector clocks. Capability grants merge via geometric CRDT operations. The intersection number is computed from eventually-consistent state. This requires the Cliffy protocols geometric CRDT layer.

---

## Design Principles (Preserved Across All Directions)

1. **No network service.** Schubert remains a library. Deployment patterns (daemon, sidecar, plugin) are the caller's choice.

2. **Identity is external.** Schubert never authenticates. It authorizes against identities provided by the caller.

3. **Mathematics is the authority.** Every access decision has a geometric basis. No heuristic rules, no pattern matching, no ML models.

4. **Synchronous by default.** The core API is synchronous. Async wrappers can be built externally.

5. **Embeddable.** A single dependency. Compiles anywhere Rust compiles. Suitable for kernels, embedded systems, browsers.

---

*Schubert 0.1.0 — May 2026*
