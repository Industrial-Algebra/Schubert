# Schubert — Agent Hand-Off

**Project:** Quantitative access control via Schubert calculus
**Location:** `/home/elliotthall/working/industrial-algebra/Schubert`
**Date:** May 2026
**Status:** Foundation complete (0.1.0). IA-conformant. Karpal-integrated. AGPL-3.0 licensed. Ready for extension.

---

## What Is Schubert?

A Rust library that replaces boolean allow/deny with **geometric access control**. Capabilities are Schubert conditions on a Grassmannian Gr(k,n). Access is granted when the intersection is non-empty, and the intersection number tells you exactly how many valid configurations exist.

The killer feature: **impossibility detection**. When σ₂·σ₁₁ = 0 in Gr(2,4), the conditions are dimensionally compatible but geometrically impossible to satisfy simultaneously. Traditional boolean AND checks would approve. Schubert catches it.

Schubert is **not** an authentication system, a network service, or a replacement for OAuth/OIDC. It's a library you embed alongside existing identity infrastructure.

---

## Current State

### Tests: 30 passing (with all features), 0 failing
```
unit tests:     30 passed (controller, composition, phantom, proof, parallel)
doc tests:       7 passed
examples:        3 compile and run
clippy:          0 warnings (all feature combos)
```

### Feature Gated
- `std` (default) — HashMap, SystemTime, thread-safe audit
- `serde` — Serialize/Deserialize on 11 key types
- `karpal` — `schubert::proof` module with compile-time verification
- `parallel` — Batch operations: `check_batch`, `stability_batch`, `compose_batch`

### Implemented
- `AccessController` — main entry point. Create principals, register/grant/revoke capabilities, check access. Batch operations: `check_batch`, `stability_batch`, `compose_batch` (requires `parallel` feature)
- `Capability` — Schubert condition with partition, kind (ReadLike/WriteLike/AdminLike/Custom), label, description
- `Principal` — wraps amari `Namespace`. Identity is external (string ID)
- `AccessDecision` — `Granted{n, path} | Impossible{conflicting} | Denied | Underconstrained{dim}`
- `ComputationPath` — LittlewoodRichardson, Localization, Tropical, Matroid
- Operadic composition — `compose()`, `are_composable()`
- Stability analysis — `analyze_stability()`, `stable_capabilities_at()`, wall-crossing phase diagrams
- Audit — pluggable `AuditSink` trait, `InMemoryAudit`, `DecisionRecord`
- Examples: rbac (Kubernetes roles), api_gateway (OAuth scope conflict), row_security (multi-tenant DB)
- **Karpal integration** — `proof` module with Proven wrappers, Property hierarchy, Rewrite rules, law checks
- Docs: README, module-level docs for all 10 modules, method docs on every public function, `docs/ROADMAP.md`, `docs/surreal-trust-levels.md`

### Fundamental Checks Verified
- σ₁⁴ = 2 in Gr(2,4) ✅
- σ₂·σ₁₁ = 0 (geometric zero → `Impossible`) ✅
- Overconstrained → `Denied` ✅
- Composition multiplicity > 0 ✅
- Revoke + holds ✅

---

## Project Architecture

```
Schubert/
├── Cargo.toml              # Depends on: amari-enumerative (path dep), thiserror, optional serde, optional karpal-proof
├── rust-toolchain.toml      # Nightly channel, rustfmt + clippy (IA ecosystem standard)
├── README.md               # Full mathematical background, API reference
├── src/
│   ├── lib.rs              # Re-exports, module docs, feature flag docs
│   ├── controller.rs       # AccessController — main entry point
│   ├── capability.rs       # Capability, CapabilityId, CapabilityKind
│   ├── principal.rs        # Principal — wraps amari Namespace
│   ├── decision.rs         # AccessDecision, ComputationPath
│   ├── composition.rs      # compose(), are_composable(), CompositionResult
│   ├── stability.rs        # analyze_stability(), TrustLevel, StabilityReport
│   ├── audit.rs            # AuditSink trait, InMemoryAudit, DecisionRecord
│   ├── phantom.rs          # Re-exports of amari phantom types for compile-time verification
│   ├── proof.rs            # Karpal integration: Proven, Property, Rewrite, law checks
│   └── error.rs            # SchubertError enum, Result alias
├── examples/
│   ├── rbac.rs             # Kubernetes 4-role model
│   ├── api_gateway.rs      # OAuth scope intersection with conflict detection
│   └── row_security.rs     # Multi-tenant row-level security
└── docs/
    ├── ROADMAP.md           # 14 speculative directions across 3 time horizons
    └── surreal-trust-levels.md  # Deep dive on surreal trust (Amari 0.22/0.23)
```

### Dependency Boundary
Schubert depends **only** on:
- `amari-enumerative` (path: `../amari/amari-enumerative`, v0.22)
- `thiserror` (v2)

Optional dependencies (behind feature gates):
- `serde` (v1, optional) — serialize/deserialize controller state

No tokio, no async runtime, no network stack. Synchronous API. Embeddable anywhere Rust compiles.

### Karpal Proof Integration (`karpal` feature)

Schubert integrates [`karpal-proof`](https://github.com/Industrial-Algebra/Karpal) (v0.3.0)
for compile-time verification of access control invariants:

| Feature | What It Provides |
|---------|-----------------|
| `Proven<IsValidCapability, Capability>` | Proof that a capability's partition is valid |
| `Proven<IsFiniteAccess, AccessDecision>` | Proof that an access check is finite (transverse) |
| Property hierarchy | `IsAdminLike: Implies<IsWriteLike>: Implies<IsReadLike>` |
| `Rewrite<GrantSeqAB, GrantSeqBA, _>` | Type-level proof that grant order is commutative |
| `law::check_*` | Runtime verification of grant idempotency, revoke identity, access idempotency |

Usage:
```rust
use schubert::proof::{IsValidCapability, prove_capability};

let cap = Capability::new("read", "Read", vec![1], CapabilityKind::ReadLike);
let proven = cap.prove((2, 4))?;  // validates partition, returns Proven<IsValidCapability, Capability>

// Derive property: admin capability implies write capability
use schubert::proof::IsAdminLike;
use karpal_proof::Proven;
let admin: Proven<IsAdminLike, Capability> = unsafe { Proven::axiom(admin_cap) };
let write: Proven<IsWriteLike, Capability> = admin.derive();  // compile-time safe
```

---

## Key Design Decisions

### 1. No Dual Storage
`Principal` wraps `Namespace` directly. No duplicate `granted: Vec<Capability>`. Capability metadata (labels, descriptions, kinds) lives in the controller's capability registry. This was the main deviation from earlier patterns — it keeps storage canonical (amari's `Namespace` is the source of truth) and avoids synchronization bugs.

### 2. Identity Is External
`PrincipalId` is a newtype over `String`. Schubert never authenticates. Map `PrincipalId` to your JWT subject, OAuth client_id, or database user ID. This makes the library embeddable in any identity infrastructure.

### 3. holds() Takes &str
`principal.holds("read:data")` — not `&CapabilityId`. The API is string-based for ergonomics. The namespace internally uses amari `CapabilityId` (Arc<str>).

### 4. Amari v0.22 API Mapping
Schubert wraps amari's types, not re-exports them. Key mappings:
- Our `Capability` → amari `Capability` (adds description, kind)
- Our `Principal` → amari `Namespace` (adds PrincipalId, created_at)
- Our `check()` → `SchubertCalculus::multi_intersect()` + hold verification
- Our `compose()` → `ComposableNamespace` + `composition_multiplicity()`
- Our `analyze_stability()` → `WallCrossingEngine` + `StabilityCondition`

### 5. Audit Is Fire-and-Forget
`AuditSink::record()` errors are silently swallowed. Audit cannot become a denial-of-service vector. The sink receives decisions after they're made.

### 6. No ShaperOS References
The project started as a ShaperOS extraction, but all references have been removed. The current codebase is a clean standalone library.

---

## Build & Test

```bash
cd /home/elliotthall/working/industrial-algebra/Schubert

# Build (zero warnings)
cargo build

# Run all tests
cargo test

# Run unit tests only
cargo test --lib

# Run examples
cargo run --example rbac
cargo run --example api_gateway
cargo run --example row_security

# Check docs
cargo doc --open
```

---

## Where to Continue

### Immediate (Low Risk, High Value)

**1. Computation Path Selection** (ROADMAP #1)
Current `check_with_path()` ignores the path parameter. Expose amari's 4 computation paths (LR, localization, tropical, matroid) and implement auto-routing based on Grassmannian size and class count.

**2. Serialization** (ROADMAP #2) — ✅ PARTIALLY DONE
`serde` feature gate added. Derives on all key types. `Principal.namespace` and `StabilityReport.walls` are skipped (external types don't impl serde). Remaining: roundtrip test, `AccessController` serde integration.

**3. Policy Language** (ROADMAP #3)
A declarative format (TOML or YAML) for defining capabilities, principals, and grants. Parse at startup, validate against the Grassmannian.

**4. Surreal Trust Levels** (ROADMAP #12)
Depend on `amari-surreal` — replace `TrustLevel(f64)` with `SurrealTrust(RationalSurreal)`. Enable exact rational trust, infinitesimal policies with epsilon, and surreal-valued phase diagrams. See `docs/surreal-trust-levels.md` for the full expansion.

### Medium-Term (Research-Adjacent)

**5. WebAssembly Target**
Make `amari-enumerative` wasm-compatible for the subset Schubert uses. Enable browser-based access control.

**6. Multi-Grassmannian Controllers**
Cross-domain access: a principal in Gr(2,4) accessing a resource in Gr(3,6) requires a morphism between Grassmannians.

**7. Proof-Carrying Capabilities** — ✅ KARPAL INTEGRATED
Capabilities can be proven valid at compile time via `Proven<IsValidCapability, Capability>`.
Property hierarchy for CapabilityKind via `Implies`. Rewrite rules for policy transformation.
Runtime law checking for access control invariants. Full cryptographic capability tokens
still future work.

### Far-Term (Speculative)

**8. Surcomplex Configuration Counting**
Complex-weighted enumeration. Geometric phase encodes correlations between access paths.

**9. Constitutional Verification**
Machine-checked proofs that capability grants satisfy Schubert calculus axioms (when karpal-proof/karpal-verify ship).

**10. Distributed CRDT Composition**
Operadic composition over eventually-consistent state using geometric CRDT operations.

---

## Amari Ecosystem Context

Schubert lives in the Industrial Algebra ecosystem at `/home/elliotthall/working/industrial-algebra/`:

| Project | What It Provides | Schubert Relevance |
|---------|-----------------|-------------------|
| **amari** (23 crates, 201K LOC) | Core math library | `amari-enumerative` — direct dependency |
| **amari-enumerative** | Schubert calculus, LR, tropical, matroids, operads, wall-crossing | The math engine |
| **amari-surreal** (v0.22+) | Dyadic surreals, RationalSurreal (v0.23), epsilon | Surreal trust levels |
| **amari-surcomplex** (v0.23 wip) | Exact rational complex arithmetic | Complex configuration counting |
| **amari-cgt** | Combinatorial game theory | Game-theoretic capability semantics |
| **Minuet** | Holographic memory toolkit | Future holographic access control |
| **ShaperOS** | Geometric operating system | Source of the Schubert calculus access model (now independent) |

The amari source is at `/home/elliotthall/working/industrial-algebra/amari/`. The v0.23 surcomplex worktree is at `amari/.worktrees/amari-0.23-rational-surcomplex/`. The main branch is still at v0.22 (`amari-surreal` and `amari-cgt` only). The `docs/amari-surcomplex-0.23-planning` and `feature/amari-0.23-rational-surcomplex` branches contain the 0.23 design docs and implementation.

The IA-MCP server (`ia-mcp`) indexes the ecosystem — 10 libraries, ~10,300 API items. Use it to explore amari's API surface:
```bash
# In pi, use the mcp tool to query amari types
```

---

## Conventions

- **Rust edition 2021**, MSRV 1.75, **nightly toolchain** (IA ecosystem standard)
- **`#![warn(missing_docs)]`** — every public item must be documented (currently zero warnings)
- **`#![warn(clippy::all)]`** — zero clippy warnings across all targets
- **Tests use Gr(2,4)** — the standard Grassmannian for access control. All fundamental checks are verified here.
- **Error types are structured** — every error variant carries contextual fields (IDs, partitions, Grassmannian parameters)
- **No unsafe code** — pure safe Rust (one exception: `InMemoryAudit` in `no_std` mode)
- **Feature gates are additive** — never break existing API:
  - `std` (default) — enables HashMap, SystemTime, Mutex-backed audit
  - `serde` — enables Serialize/Deserialize on key types
  - `karpal` — enables the proof module with Proven, Rewrite, law checks
  - `parallel` — enables batch operations via rayon: `check_batch`, `stability_batch`, `compose_batch`
- **Phantom types** from `amari_enumerative::phantom` are re-exported via `schubert::phantom` for compile-time verification. Compatible with `karpal_proof::Proven` for proof-carrying capabilities.
- **`no_std` compatibility** — scaffolded with `std` feature gate. HashMap → BTreeMap, Mutex → single-threaded, now_millis() → 0. Full `no_std` requires `amari-enumerative` without `std`.

---

## Quick Recipes

### Adding a New Computation Path
1. Add variant to `ComputationPath` in `decision.rs`
2. Map it in `controller.rs` `check_with_path()` to the corresponding amari method
3. Add a test verifying the path produces the same result as LR for σ₁⁴ = 2

### Adding a New CapabilityKind
1. Add variant to `CapabilityKind` in `capability.rs`
2. Document its typical codimension
3. Update the `CapabilityKind` table in `README.md`

### Adding Serialization
1. ✅ `serde` feature added to `Cargo.toml`
2. ✅ Derive `Serialize, Deserialize` on `Capability`, `CapabilityId`, `CapabilityKind`, `PrincipalId`, `Principal` (namespace skipped), `AccessDecision`, `ComputationPath`, `CompositionResult`, `TrustLevel`, `StabilityBreakpoint`, `StabilityReport` (walls skipped), `DecisionRecord`
3. Add roundtrip test: serialize → deserialize → same decisions
4. ✅ Gated behind `#[cfg(feature = "serde")]`

---

*Hand-off prepared May 2026. All tests passing. Zero warnings. Ready for the next session.*

---

## IA Ecosystem Conformance (May 2026)

Schubert now conforms to Industrial Algebra ecosystem conventions:

| Convention | Status |
|---|---|
| `rust-toolchain.toml` (nightly + rustfmt + clippy) | ✅ Added |
| `serde` behind optional feature gate | ✅ Added |
| Phantom types from amari-enumerative re-exported | ✅ `schubert::phantom` |
| `std`/`no_std` feature scaffolding | ✅ Gate on HashMap, Mutex, SystemTime |
| `#![warn(missing_docs)]` + `#![warn(clippy::all)]` | ✅ Zero warnings |
| Custom error type via `thiserror` | ✅ `SchubertError` (11 variants) |
| Karpal `Proven` + `Rewrite` integration prep | ✅ Phantom types ready; Karpal dep TBD |

---

## Licensing

Schubert is dual-licensed under:

- **GNU Affero General Public License v3 (AGPL-3.0-only)** — see `LICENSE`
- **Commercial License** — see `LICENSE-COMMERCIAL`

SPDX-License-Identifier: AGPL-3.0-only
Copyright (C) 2026 Industrial Algebra. All rights reserved.

### License Headers
All source files carry SPDX headers:
```rust
// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: AGPL-3.0-only
```

### Contributor License Agreement
Contributors must sign a CLA before pull requests can be merged.
See `CONTRIBUTING.md` for details.

### Key Licensing Files
| File | Purpose |
|------|--------|
| `LICENSE` | Full AGPL v3 text with Industrial Algebra copyright notice |
| `LICENSE-COMMERCIAL` | Commercial licensing terms and contact information |
| `CONTRIBUTING.md` | CLA requirements and contribution process |
| `Cargo.toml` | `license = "AGPL-3.0-only"` in SPDX format |
