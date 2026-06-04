# Schubert — Quantitative Access Control

> "How many ways can this principal access this resource?"

Schubert replaces boolean allow/deny with **geometric access control** built on Schubert calculus. Capabilities are Schubert conditions on a Grassmannian — access is granted when the intersection is non-empty, and the intersection number tells you exactly how many valid configurations exist.

## Why Geometry?

Traditional access control answers one question: "allowed or denied?" Schubert answers four:

| Question | Traditional | Schubert |
|----------|-------------|----------|
| Is access allowed? | Yes/no | Yes/no |
| **How many ways?** | N/A | Exact integer count |
| **Is the policy over/under-constrained?** | N/A | PositiveDimensional / Denied |
| **Are conditions geometrically impossible?** | N/A | Impossible (σ₂·σ₁₁ = 0) |
| **Can policies be composed?** | Manual AND | Operadic composition with multiplicity |
| **How stable under trust degradation?** | N/A | Wall-crossing phase diagram |

The killer feature: **impossibility detection**. When `σ₂·σ₁₁ = 0` in Gr(2,4), the conditions are dimensionally compatible but geometrically impossible to satisfy simultaneously. A traditional boolean AND check would approve. Schubert catches it.

## Mathematical Foundation

### Grassmannians

A Grassmannian Gr(k,n) is the space of all k-dimensional subspaces of an n-dimensional vector space. Its dimension is k(n−k). Access control operates within this space:

- Each **principal** occupies a position in Gr(k,n) — a particular subspace
- Each **capability** is a Schubert condition — a constraint that reduces the accessible subspace
- **Access** is granted when the intersection of all conditions is non-empty
- The **intersection number** counts the valid configurations

### Schubert Calculus

Schubert classes σ_λ are cohomology classes on the Grassmannian, indexed by integer partitions λ = (λ₁ ≥ λ₂ ≥ ... ≥ λₖ). Each class has a codimension equal to the sum of its parts.

The intersection product σ_λ · σ_μ is computed via the Littlewood-Richardson rule:

```
σ_λ · σ_μ = Σ c^ν_{λ,μ} σ_ν
```

where c^ν_{λ,μ} are the Littlewood-Richardson coefficients — non-negative integers that count valid configurations.

### Key Examples in Gr(2,4)

The Grassmannian Gr(2,4) has dimension 4 and is the standard configuration for RBAC:

| Product | Result | Meaning |
|---------|--------|---------|
| σ₁⁴ | 2 | Four read-like conditions yield 2 valid configurations |
| σ₂² | 1 | Two write-like conditions yield exactly 1 configuration |
| σ₂·σ₁₁ | 0 | **The geometric zero**: individually valid, geometrically impossible |
| σ₂₂ | 1 | The point class — a single admin configuration |
| σ₁·σ₂₁ | 1 | A read + admin combo yields exactly 1 configuration |

## Documentation

- [Getting Started](docs/guide/getting-started.md) — installation and first controller
- [Concepts](docs/guide/concepts.md) — mathematical foundation
- [Architecture](docs/guide/architecture.md) — module map and data flow
- [Cookbook](docs/guide/cookbook.md) — integration recipes (OAuth, JWT, databases)
- [Feature Flags](docs/guide/feature-flags.md) — available optional features
- [ROADMAP](docs/ROADMAP.md) — completed and speculative directions
- [CHANGELOG](CHANGELOG.md) — version history

## Quick Start

```rust
use schubert::{AccessController, Capability, CapabilityKind, AccessDecision};

let mut acl = AccessController::new(2, 4)?;

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
        println!("Granted with {} configurations via {:?}", configurations, path);
    }
    AccessDecision::Impossible { conflicting } => {
        println!("Geometrically impossible: {:?}", conflicting);
    }
    AccessDecision::Denied => println!("Denied"),
    AccessDecision::Underconstrained { dimension } => {
        println!("Underconstrained (dimension {})", dimension);
    }
}
```

## Core Types

### AccessDecision

The output of every access check — never just a boolean:

```rust
pub enum AccessDecision {
    /// Access granted with N valid configurations
    Granted { configurations: u64, path: ComputationPath },
    /// Dimensional match but geometric zero (σ₂·σ₁₁ = 0)
    Impossible { conflicting: Vec<CapabilityId> },
    /// Too many conditions (codim > dimension)
    Denied,
    /// Too few conditions (positive-dimensional variety)
    Underconstrained { dimension: usize },
}
```

### Capability

Schubert conditions with human-readable metadata:

```rust
pub struct Capability {
    pub id: CapabilityId,
    pub label: String,
    pub description: String,
    pub partition: Vec<usize>,  // e.g., [1] = σ₁, [2,1] = σ₂₁
    pub kind: CapabilityKind,   // ReadLike, WriteLike, AdminLike, Custom
}
```

### CapabilityKind

Semantic hints for policy reasoning:

| Kind | Typical Codimension | Example |
|------|--------------------|---------|
| `ReadLike` | 1 | `read:data` = σ₁ |
| `WriteLike` | 2 | `write:data` = σ₂ |
| `AdminLike` | ≥ dimension | `admin:*` = σ₂₂ (point) |
| `Custom` | Any | Arbitrary partition |

## Grassmannian Selection

| Gr(k,n) | Dim | Capacity (distinct conditions) | Use Case |
|---------|-----|-------------------------------|----------|
| Gr(1,2) | 1 | 1 | Simple binary access |
| Gr(1,3) | 2 | 2 | Read/write on one resource |
| **Gr(2,4)** | **4** | **4** | **Standard RBAC (recommended)** |
| Gr(2,5) | 6 | 5 | RBAC with 5 distinct conditions |
| Gr(3,6) | 9 | 6 | Complex multi-tenant |
| Gr(4,8) | 16 | 8 | Enterprise policy space |

The Grassmannian dimension k(n−k) determines the total codimension budget. Each capability consumes codimension equal to the sum of its partition. When total codimension equals the dimension, you get exact configurations (Finite). When it exceeds, access is denied. When it falls short, the policy is underconstrained.

## Operadic Composition

Compose two principals by consuming one capability from each as the interface:

```rust
let result = compose(&acl, &producer, "output:data", &consumer, "input:data")?;
// result.multiplicity — the pushforward degree
// result.retained_capabilities — non-interface capabilities from both
```

This models service chaining: service A produces output that service B consumes. The composed access policy is the intersection after composition, with multiplicity reflecting how many configurations survive the composition.

## Stability Analysis

Understand how access degrades as trust decreases via wall-crossing phase diagrams:

```rust
let report = analyze_stability(&acl, &principal)?;

// report.phase_diagram — breakpoints where capabilities become unstable
// report.walls — individual stability walls
// report.total_capabilities — total count

// Check what remains stable at a specific trust level
let stable = stable_capabilities_at(&acl, &principal, TrustLevel::new(0.5))?;
```

Capabilities with higher codimension/dimension ratios are more sensitive to trust degradation. The wall-crossing engine computes exactly at what trust levels each capability becomes unstable.

## Audit Trail

Pluggable audit sink for recording all access decisions:

```rust
use schubert::audit::{AuditSink, DecisionRecord, InMemoryAudit};

let audit = InMemoryAudit::new();
acl.set_audit_sink(Box::new(audit));

// Every check() call now records a DecisionRecord
// { principal, capabilities, decision, timestamp }
```

Implement `AuditSink` for your own storage backend (database, log stream, event bus).

## Feature Flags

| Feature | What It Enables |
|---------|----------------|
| `std` (default) | HashMap, SystemTime, thread-safe audit via Mutex |
| `serde` | Serialize/Deserialize on all key types + JSON I/O |
| `karpal` | Compile-time verification via `karpal_proof::Proven` + Rewrite |
| `parallel` | Batch operations via rayon: `check_batch`, `stability_batch` |
| `policy` | Declarative TOML policy language |
| `wasm` | wasm-bindgen JS bindings for browser-based access control |

## Computation Paths

Four engines for computing Schubert intersections, selectable via `check_with_path()`:

| Path | Engine | Best For |
|------|--------|----------|
| `LittlewoodRichardson` | Classical LR rule | Small Gr(k,n), exact results |
| `Localization` | Atiyah-Bott fixed-point | Large Gr(k,n), many classes |
| `Tropical` | Tropical intersection theory | Fast approximate counts |
| `Matroid` | Matroid independence check | Polynomial-time impossibility detection |

Auto-routing via `check_auto()`: uses LR for Gr(k,n≤8) and localization for larger spaces.

## Policy Language (`policy` feature)

Define access control policies declaratively in TOML:

```toml
# examples/policies/rbac.toml
[grassmannian]
k = 2
n = 4

[capabilities.read_pods]
partition = [1]
kind = "ReadLike"
label = "Read pods"

[principals.alice]
grants = ["read_pods"]
```

```rust
let acl = AccessController::from_policy_toml(&toml_str)?;
// Roundtrip: export back to TOML
let toml = acl.to_policy_toml()?;
```

The policy is validated at parse time — partitions must fit in the Grassmannian,
be weakly decreasing, and all principal grants must reference defined capabilities.

## WebAssembly (`wasm` feature)

Embed Schubert in the browser via `WasmController`:

```js
import { WasmController } from './schubert.js';

const acl = new WasmController(2, 4);
acl.register_capability("read", "Read", [1], "ReadLike");
const alice = acl.create_principal("alice");
acl.grant(alice, "read");
const result = acl.check(alice, ["read"]);
// result.tag = "underconstrained", result.dimension = 3
```

Compiles to `wasm32-unknown-unknown`. Audit sink requires `std` (not available on wasm).

## Context-Aware Access

Extend access checks with resource scoping and time-aware trust:

```rust
use schubert::AccessContext;

// Resource-scoped: also checks "read/doc/42" if registered
let ctx = AccessContext::for_resource("doc/42");
let decision = acl.check_with_context(&alice, &["read"], &ctx)?;

// Time-aware: trust decays over 2 years, scaling config counts
let ctx = AccessContext::at_time(future_timestamp);
let decision = acl.check_with_context(&alice, &["admin"], &ctx)?;
```

## Multi-Grassmannian Control

Manage access across multiple Grassmannian domains:

```rust
use schubert::MultiController;

let mut mc = MultiController::new();
let gr24 = mc.add_domain(2, 4)?;  // Standard RBAC domain
let gr36 = mc.add_domain(3, 6)?;  // Multi-tenant domain

mc.register_in_domain(Capability::new("read", "Read", vec![1], …), &gr24)?;
let alice = mc.create_principal("alice", &gr24)?;
mc.grant_in_domain(&alice, "read", &gr24)?;

// Cross-domain: check if Gr(2,4) capability works in Gr(3,6)
mc.check_cross_domain(&alice, &["read"], &gr24, &gr36)?;
```

## Examples

```bash
# Kubernetes RBAC with 4 roles + stability analysis
cargo run --example rbac

# OAuth scope intersection with geometric conflict detection
cargo run --example api_gateway

# Multi-tenant row-level security
cargo run --example row_security

# Policy-driven access from TOML file
cargo run --example policy_loader --features policy

# Multi-Grassmannian cross-domain access
cargo run --example cross_domain

# Context-aware decisions with resource scoping
cargo run --example context_aware
```

## Dependencies

- `amari-enumerative` v0.22 — Schubert calculus, intersection theory, wall-crossing
- `thiserror` v2 — Error derive macros

Optional (behind feature gates):
- `serde` + `serde_json` — Serialization and JSON I/O
- `karpal-proof` v0.3 — Compile-time proof verification
- `rayon` — Parallel batch operations
- `toml` — Policy language parsing
- `wasm-bindgen` + `js-sys` — WebAssembly bindings

No async runtime. No network stack. Embeddable in any Rust project.

## What Schubert Is Not

- An authentication system (identity is external — bring your own JWT/OAuth/OIDC)
- A network service or daemon (library only — embed it)
- A replacement for existing IAM (compatible alongside — supplements boolean checks with geometric analysis)
- A key-value store or database (the audit trail is a trait — plug in your own persistence)

## License

Schubert is dual-licensed:

- **GNU Affero General Public License v3 (AGPL-3.0)** — see [LICENSE](./LICENSE)
- **Commercial License** — for proprietary use, see [LICENSE-COMMERCIAL](./LICENSE-COMMERCIAL)

In summary:

| Use Case | License |
|----------|--------|
| Open-source (AGPL-compatible) projects | AGPL v3 — free |
| Proprietary / closed-source software | Commercial license required |
| SaaS / hosted service (without releasing modifications) | Commercial license required |
| Evaluation, research, personal projects | AGPL v3 — free |

For commercial licensing inquiries: <license@industrial-algebra.org>

Copyright © 2026 Industrial Algebra. All rights reserved.
