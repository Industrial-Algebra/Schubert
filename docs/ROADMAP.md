# Schubert — Directions

> **v0.4.0 Snapshot** — All 14 core roadmap items complete, plus the v0.4.0
> consumer-driven package: multi-capability `GrantToken`s, `schubert::axum`,
> `KeyStore`, binary wire format, `check_single`, and the
> `schubert-tsukoshi` npm package. Karpal/Minuet upgraded to Apache-2.0.
> Formal mapping substantiated via distributed game sync design. 14 Proserpina
> critique findings addressed.
> See [CHANGELOG.md](../CHANGELOG.md) for version history.

**Version:** 0.4.0 — Consumer-driven grants. Apache-2.0 throughout.
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
- ✅ Multi-capability grant tokens + `GrantVerifier` (v0.4.0)
- ✅ `schubert::axum` bearer extractor (v0.4.0)
- ✅ `KeyStore`, binary wire format, `check_single` (v0.4.0)
- ✅ `schubert-tsukoshi` npm package (v0.4.0)

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
See `docs/surreal-trust-levels.md` for the expansion (rational/infinitesimal
layers, the analytic φ(t) generalization).

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

## Shipped (v0.4.0)

### 15. schubert-tsukoshi — Pure TypeScript Access Control — ✅ DONE (v0.4.0)

**Shipped** as [`@industrialalgebra/schubert-tsukoshi`](https://www.npmjs.com/package/@industrialalgebra/schubert-tsukoshi):
zero-dependency core (LR tables for Gr(2,4)/Gr(3,6)/Gr(4,8), impossibility
detection), an Ed25519 `crypto` subpath with a Rust-compatible wire format
(tokens interop both directions), and a `protocols` subpath with `GrantCRDT`
(replicated grant set over cliffy-tsukoshi's `VectorClock`).

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

**Scope:** ~1 week of focused work. Published as `@industrialalgebra/schubert-tsukoshi` on npm (the canonical IA scope; cliffy-tsukoshi and amari-wasm will migrate there during their refactors).

**Reference:** [cliffy-tsukoshi](https://github.com/justinelliottcobb/Cliffy/tree/main/cliffy-tsukoshi) —
the pattern this follows (pure TS extraction of geometric math from a Rust
framework, with distributed protocols).

### 16. Consumer-Driven API Polish (from Ijima Integration) — ✅ DONE (v0.4.0)

**Shipped in full** (all six items): `axum` module (`AuthPrincipal` bearer
extractor), multi-capability `GrantToken`/`GrantVerifier` with geometric
containment (`may`), `crypto::KeyStore` (0600, atomic load-or-create), binary
wire format on both token types, `from_seed`/`public_key_hex`, and
`check_single`. Dominic's federation routing (M3) builds on the grant
machinery — the second consumer.

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

**4. Multi-capability tokens (grants)**
Tokens currently carry one capability. Ijima's pi integration needs 4–6
capabilities simultaneously and currently juggles an env-bundle of one-cap
tokens. Dominic's federation orchestration has the same pain at scale.
Full requirements in
[`handoff-multi-capability-tokens.md`](handoff-multi-capability-tokens.md)
(PR #29). Design: a `Grant` carries `Vec<CapabilityId>`, signed once,
verifiable per-capability. Geometrically, a grant is a **subvariety** of
Gr(k,n) — the union of the granted Schubert varieties — and `may(cap)` asks
whether the cap's Schubert variety is contained in that subvariety.
Singleton grants `[cap]` are backward-compatible with today's one-cap tokens.
**Relationship to #18:** within a single Grassmannian, grant containment is
set membership (v0.4.0). The flag variety embedding (#18) generalizes this
to cross-domain grants and upgrades set membership to geometric containment.
See the analysis below.

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

## Near-Term (v0.5.0) — Grant Lifecycle (consumer-driven)

### 20. Grant Lifecycle — expiry & nonce, grant-aware revocation, policy-issuance linkage

**Origin:** The 0.4.0 consumer wave (Ijima → Dominic) shipped grants, but
three lifecycle gaps are now pressing for the next consumers: **Wallace
extensions** (capability-gated extension packages), **Dominic federation**
(peer grants across instances), and **Ijima** (peer grants + the deferred
`capability_policy_ref`). Each item below is verified against the 0.4.0 code.

**1. `GrantToken` expiry & nonce (Ijima-driven — full spec below)**

**Motivation (field friction, from the first real consumer):** Ijima
v0.2.0 hardened its deployment path and hit two `GrantToken` gaps that
policy-layer features cannot cover:

1. **No token-carried expiry.** A bearer is valid until the issuer key
dies. Ijima built a store-backed revocation list for *incidents*
(kill a leaked token now — `docs/adr/token-revocation.md` in the Ijima
repo), but *routine* deprovisioning (service-feed rotation, principal
offboarding) wants TTLs the **verifier can check standalone** — critical
for the v0.3 federation shape, where satellite instances verify grants
without phoning home to a controller.
2. **No nonce/jti.** `issue_grant` signs `(principal, capabilities,
issuer_key)` — so re-issuing the same grant from the same seed yields a
**byte-identical bearer**. A revoked token therefore cannot be cleanly
re-issued: the revocation hash kills the re-issue too. A random nonce
makes every issuance distinct.

**Relationship to existing work:** the policy layer already has temporal
access control (`Capability::expires_at`, `check_temporal()`, trust
decay — item #8), but that requires the `AccessController` at check
time. This item is the **crypto layer**: standalone, proof-carrying,
verifier-side only.

**Spec sketch:**

- `GrantToken` gains `expires_at_unix: Option<u64>` (None = never) and
  `nonce: [u8; 16]` (random at issue).
- Both fields are covered by the signature: extend the canonical signing
  message in `issue_grant` and `GrantVerifier::verify` identically.
- The nonce is **not** part of the canonical capability sort — sort stays
  `(partition, id)`; distinctness comes from the signed nonce alone.
- Wire format: extend `GrantToken::to_bytes`/`from_bytes`. Breaking change
to the blob layout is acceptable in a 0.x minor (Ijima re-mints; no
external bearers exist beyond it).
- `GrantVerifier::verify_at(grant, now_unix)` — deterministic,
clock-injected variant — with `verify(grant)` delegating via
`SystemTime::now()`. Expired ⇒ the same error class as a bad signature.
- Issuer surface: `issue_grant` defaults `nonce = random`,
  `expires_at = None`; add `issue_grant_with_expiry(principal, caps,
  expires_at)` (and/or a builder) for explicit control. Deterministic
  issuance for tests: allow injecting the nonce.
- **tsukoshi parity:** mirror in
  `@industrialalgebra/schubert-tsukoshi` (crypto subpath), including the
  cross-language fixture test (Rust-issued ⇒ TS-verified, both expiry and
  nonce paths).

**Tests:**

- Round trip: future expiry verifies, past expiry rejected,
  `verify_at` determinism (same grant, injected clocks).
- Distinctness: two `issue_grant` calls with identical inputs produce
  different bytes, both verify (nonce works).
- Tamper: flipping expiry or nonce bytes ⇒ signature failure.
- Sort stability: capability ordering unchanged by nonce randomness.
- Cross-language fixture (tsukoshi).

**References:** Ijima `docs/adr/token-revocation.md` (complementary
rationale — revocation = incidents, expiry = deprovisioning), Ijima
`docs/adr/grant-token-migration.md` (consumer context), Schubert
`docs/handoff-multi-capability-tokens.md` (GrantToken origin).

**Interaction with consumer revocation** (per Anima PULSE 2026-08-17
rec #5 — the half-page scoping it asked for): Ijima already runs a
store-backed revocation list checked at verify. The composing rules the
0.5 design must not leave undefined:

- **Verify order:** consumers check revocation-or-expiry as one rejection
  step — either answer is "dead", and which one fired is telemetry, not
  semantics. `verify_at` returning a distinct error class for expiry lets
  callers distinguish without a second API.
- **Clock skew:** expiry is a wall-clock comparison; satellites may drift.
  Recommended: document a skew tolerance (e.g. ±30s leeway on
  `expires_at_unix` comparisons) or leave skew policy to the caller via
  the injected clock in `verify_at` — either way, say which.
- **Renewal = re-issue:** there is no renewal mutation; a rotated grant is
  a fresh `issue_grant` (new nonce ⇒ distinct bearer), with the old one
  expiring or being revoked. The nonce (this item) is what makes that
  clean — document the pattern rather than adding a renewal API.

**Scope:** ~2–4 days including tsukoshi parity.


**2. Grant-aware CRDT revocation**
`CrdtGrant` (crdt.rs) tracks a single `CapabilityId` — pre-`GrantToken`
machinery. Multi-participant Wallace needs revoke-grant → converges across
session participants; Ijima needs peer revocation; Dominic needs peer
revocation converging across federation state. Either upgrade `CrdtGrant` to
carry a grant id, or add revocation registries (tombstones) keyed by grant
hash. `GrantCRDT` (tsukoshi) will need the parallel treatment.
**Resolved: [ADR-0002](adr/0002-grant-crdt-revocation.md) — tombstone registry
(grow-only set, union merge) keyed by the #20.1 issuance nonce; implemented in
v0.5.0 (Rust `revoke_grant`/`is_grant_revoked`, tsukoshi
`revokeGrant`/`isGrantRevoked`).**

**3. Policy → issuance linkage**
The controller path has `from_policy_toml` (#3); grants have
`CapabilityIssuer` — but nothing connects them. The seam: policy.toml
drives *what grants may be issued* (constrained issuance), so Ijima can own
policy while principals carry grants — the capability-driven control-API
design (Dominic ROADMAP §3.5 anticipates this).
**Resolved (v0.5.0): `GrantPolicy` + `issue_grant_under_policy` (crypto × policy
features) — issuance requires an exact `(id, partition)` entitlement match;
`PolicyConfig::grants_for` is the feature-free entitlement view.**

**Scope:** item 1 ~2–4 days (incl. tsukoshi parity, spec above); items 2–3 add
~1 week. Same character as #16 — each item eliminates boilerplate a consumer
has written or is about to write. Directly validated by the
Wallace/Dominic/Ijima 2026-08 build wave.

---

## Research Directions (v0.5.0+ and Beyond)

### 17. Compositional Wall-Crossing

**Expanded:** see
[`docs/design/wall-crossing-diffusion-composition.md`](design/wall-crossing-diffusion-composition.md)
— the v0.5.0 framing via diffusion-LM composition for Quantizon (BPS
non-additivity as a candidate formalism for emergence; tropical bridge). A
KS-type composition probe (`analyze_composed_stability`) **landed in v0.5.0
development** (PR #37, merged: computes `P_A`/`P_B`/`P_C` and tests additivity
— see the [v0.5.0 sprint plan](plans/2026-08-17-v0.5.0-sprint-plan.md)).

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

**Relationship to #16.4 (multi-cap tokens):** A multi-cap grant on a single
Grassmannian is a union of Schubert varieties — containment is set membership
or codimension comparison. But a cross-domain grant (capabilities on both
Gr(k₁,n) and Gr(k₂,n)) lives naturally on the flag variety Fl(k₁, k₂, n):
"this principal holds a flag V_{k₁} ⊂ V_{k₂} ⊂ Cⁿ satisfying both domain
conditions." The flag variety embedding upgrades the v0.4.0 set-membership
semantics to true geometric containment across domains.

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
