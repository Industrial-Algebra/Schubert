# Schubert — Quantitative Access Control via Schubert Calculus

> "How many ways can this principal access this resource?"

Schubert replaces boolean allow/deny with **geometric access control** built on
Schubert calculus. Capabilities are Schubert conditions on a Grassmannian —
access is granted when the intersection is non-empty, and the intersection number
tells you exactly how many valid configurations exist.

**Killer feature:** Impossibility detection. When `σ₂·σ₁₁ = 0` in Gr(2,4), the
conditions are individually valid but geometrically impossible to satisfy
together. A boolean AND would approve. Schubert catches it. See the
[distributed game sync design](docs/design/distributed-game-sync.md) for the
concrete case: a player can't simultaneously be in combat mode and a safe zone.

## Quick Start

```rust
use schubert::{AccessController, Capability, CapabilityKind, AccessDecision};

let mut acl = AccessController::new(2, 4)?;  // Gr(2,4) — standard RBAC

acl.register_capability(Capability::new(
    "read:data", "Read data", vec![1], CapabilityKind::ReadLike,
))?;
acl.register_capability(Capability::new(
    "write:data", "Write data", vec![2], CapabilityKind::WriteLike,
))?;

let alice = acl.create_principal("alice")?;
acl.grant(&alice, "read:data")?;
acl.grant(&alice, "write:data")?;

match acl.check(&alice, &["read:data", "write:data"])? {
    AccessDecision::Granted { configurations, path } => {
        println!("Granted: {configurations} configurations via {path}");
    }
    AccessDecision::Impossible { conflicting } => {
        println!("Geometrically impossible: {conflicting:?}");
    }
    AccessDecision::Denied => println!("Denied (overconstrained)"),
    AccessDecision::Underconstrained { dimension } => {
        println!("Underconstrained (dimension {dimension})");
    }
}
```

## Why Geometry?

Traditional access control answers one question: "allowed or denied?" Schubert
answers four:

| Question | Traditional | Schubert |
|----------|-------------|----------|
| Is access allowed? | Yes/no | Yes/no |
| **How many ways?** | N/A | Exact integer (Littlewood-Richardson coefficient) |
| **Are conditions impossible together?** | N/A | `Impossible` (σ₂·σ₁₁ = 0) |
| **Can policies compose?** | Manual AND | Operadic composition with multiplicity |
| **How stable under trust degradation?** | N/A | Wall-crossing phase diagram |

## What Schubert Is Not

- **Not an authentication system** — identity is external (bring your own JWT/OAuth/OIDC)
- **Not a network service** — library only, embed it in your application
- **Not a database** — in-memory state, serialize via `serde` for persistence
- **Not a replacement for RBAC/ABAC** — supplements boolean checks with geometric analysis

## Documentation

**Online book:** [schubert.industrialalgebra.com](https://schubert.industrialalgebra.com)

- [Getting Started](docs/guide/getting-started.md) — installation and first controller
- [Concepts](docs/guide/concepts.md) — mathematical foundation
- [Architecture](docs/guide/architecture.md) — module map and data flow
- [Cookbook](docs/guide/cookbook.md) — integration recipes (OAuth, JWT, databases)
- [CLI Guide](docs/guide/cli.md) — discovery CLI for LLM agents
- [Feature Flags](docs/guide/feature-flags.md) — available optional features
- [Distributed Game Sync](docs/design/distributed-game-sync.md) — formal mapping + impossibility case
- [Threat Model](book/src/design/threat-model.md) — adversary model and security properties
- [Critique & Future Work](book/src/design/critique.md) — honest assessment
- [CHANGELOG](CHANGELOG.md) — version history
- [Literature Survey](docs/related-work-survey.md) — 10 novelty claims, zero prior work at this intersection

## Grassmannian Selection

| Gr(k,n) | Dimension | Use Case |
|---------|-----------|----------|
| Gr(1,2) | 1 | Simple binary access |
| **Gr(2,4)** | **4** | **Standard RBAC (recommended)** |
| Gr(3,6) | 9 | Complex multi-tenant |
| Gr(4,8) | 16 | Enterprise policy space |

## Core Types

### AccessDecision — never just a boolean

```rust
pub enum AccessDecision {
    Granted { configurations: u64, path: ComputationPath },
    Impossible { conflicting: Vec<CapabilityId> },  // σ₂·σ₁₁ = 0
    Denied,                                          // overconstrained
    Underconstrained { dimension: usize },           // policy too loose
}
```

### Capability — Schubert condition with metadata

```rust
pub struct Capability {
    pub partition: Vec<usize>,    // [1] = σ₁, [2] = σ₂, [2,1] = σ₂₁
    pub kind: CapabilityKind,     // ReadLike, WriteLike, AdminLike, Custom
    pub expires_at: Option<u64>,  // temporal access
}
```

## Features

| Feature | What It Enables |
|---------|----------------|
| `std` (default) | HashMap, SystemTime, thread-safe audit |
| `serde` | Serialize/Deserialize, JSON I/O |
| `policy` | TOML policy language with validation |
| `parallel` | Batch operations via rayon |
| `crypto` | Ed25519 signed capability tokens |
| `karpal` | Type-level proofs (Proven, Rewrite, law checks) |
| `karpal-verify` | SMT/Lean proof obligations, Certified trust boundary |
| `surreal` | Exact surreal trust (RationalSurreal + EpsilonPolynomial) |
| `holographic` | Minuet holographic memory integration |
| `wasm` | wasm-bindgen JS bindings for browser access control |

## Advanced Capabilities

### Operadic Composition

```rust
let result = compose(&acl, &producer, "output:data", &consumer, "input:data")?;
// result.multiplicity — how many configurations survive composition
```

### Stability Analysis

```rust
let report = analyze_stability(&acl, &principal)?;
// report.phase_diagram — trust breakpoints where capabilities become unstable
```

### Distributed CRDTs

```rust
let mut state = CrdtState::new(2, 4)?;
state.set_max_staleness(Some(30_000)); // 30 seconds
state.merge(&remote_state)?;
if !state.is_converged_with(&other_version) {
    println!("Not yet converged");
}
```

### Rate Limiting (scaled by intersection numbers)

```rust
let mut rl = RateLimiter::new(10.0, 1.0);
rl.configure_from_decision("alice", &granted_decision)?;
```

### Surreal Trust

```rust
use schubert::surreal_trust::SurrealTrust;
let trust = SurrealTrust::from_ratio(1, 2); // exact 0.5
let eps = SurrealTrust::epsilon(); // infinitesimal — provably > 0, < any rational
```

## Computation Paths

| Path | Engine | Best For |
|------|--------|----------|
| `LittlewoodRichardson` | Classical LR rule | Exact results, small Gr(k,n) |
| `Localization` | Atiyah-Bott fixed-point | Large Gr(k,n) |
| `Tropical` | Tropical intersection | Fast approximate counts |
| `Matroid` | Matroid independence | Polynomial-time impossibility detection |

## CLI — LLM Discovery Tool

```bash
cargo install schubert

schubert discover                    # compact JSON API catalog
schubert recommend                   # interactive config recommender
schubert explore --eval '{...}'      # one-shot access decision evaluator
```

## Dependencies

| Crate | Version | Required? |
|---|---|---|
| `amari-enumerative` | 0.23 | **Yes** (core — Schubert calculus engine) |
| `karpal-proof` | 0.6 | Optional (`karpal` feature) |
| `karpal-verify` | 0.6 | Optional (`karpal-verify` feature) |
| `karpal-schubert-types` | 0.6 | Optional (`karpal-verify` feature) |
| `amari-surreal` | 0.23 | Optional (`surreal` feature) |
| `minuet` | 0.5 | Optional (`holographic` feature) |

No async runtime. No network stack. Embeddable in any Rust project.

## Examples

```bash
cargo run --example rbac              # RBAC with stability analysis
cargo run --example api_gateway       # OAuth scope intersection
cargo run --example row_security      # Multi-tenant row-level security
cargo run --example cross_domain      # Multi-Grassmannian cross-domain
cargo run --example context_aware     # Resource-scoped, time-aware trust
cargo run --example policy_loader     # TOML policy-driven access
cargo run --example rate_limiter      # Intersection-number rate limiting
```

## What's New

### v0.4.0
- **`schubert::axum` module** — bearer-token `AuthPrincipal` extractor (401 vs 500 split)
- **Multi-capability grant tokens** — `GrantToken`/`GrantVerifier` with geometric containment (`may()`: write implies read, admin implies all)
- **`KeyStore`** — file-based Ed25519 seed persistence (`0600`)
- **Binary wire format** — `to_bytes()`/`from_bytes()` for `CapabilityToken` + `GrantToken`
- **`check_single()`** — set-membership fast path for per-request checks
- **`schubert-tsukoshi`** — pure-TypeScript extraction (`@industrialalgebra/schubert-tsukoshi`): zero-dep core, Rust-compatible crypto, `GrantCRDT`

### v0.3.0
- **Karpal 0.6.1 + Minuet 0.5.0** — all dependencies now Apache-2.0
- **Distributed game sync design** — formal mapping, impossibility substantiation
- **Threat model, flag structure, adversarial concerns** — security documentation
- **`karpal_compose` module** — type-level triple-composition verification
- **`HolographicStore` enum** — Simple + Sharded variants
- 14 of 22 Proserpina critique findings addressed

### v0.2.0
- **Apache-2.0 license** — AGPL network clause removed
- **Performance benchmarks** — criterion for 4 paths × 3 Grassmannians
- **Deployment example** — Axum web middleware
- **CRDT staleness gating** — `set_max_staleness()`, convergence checks

## License

Schubert is licensed under **Apache-2.0** with a Contributor License Agreement
(CLA). See [LICENSE](./LICENSE) for the full text. All contributors must sign
the [CLA](https://github.com/Industrial-Algebra/.github/blob/main/CLA.md).

Copyright © 2026 Industrial Algebra. All rights reserved.
