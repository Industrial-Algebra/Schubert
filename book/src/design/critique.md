# Critique & Future Work

> **v0.3.0 Snapshot** — This page tracks the project's intellectual honesty:
> what critiques have been raised, what's been addressed, and what remains.

## Critique Round 1 — Initial Self-Assessment (v0.1.0)

These concerns were identified during the initial development phase.

| Concern | Resolution |
|---|---|
| No CLI tooling | ✅ CLI with `discover`, `recommend`, `explore` subcommands |
| AGPL-only licensing | ✅ Apache-2.0 with CLA (v0.2.0) |
| Sparse documentation | ✅ User guide, book, API reference, cookbook |
| Feature-flag complexity | ✅ Feature flag guide with common combinations |
| Performance benchmarks missing | ✅ criterion benchmarks for 4 paths × 3 Grassmannians (v0.2.0) |
| No deployment examples | ✅ Axum middleware example (v0.2.0) |
| CRDT staleness unguarded | ✅ `set_max_staleness()`, `staleness_ms()`, `is_converged_with()` (v0.2.0) |

## Critique Round 2 — Proserpina Panel (v0.3.0)

A 5-critic LLM panel (Devil's Advocate, Methodologist, Red Team, Domain Expert,
Editor) cross-examined the README, book introduction, and mathematical concepts
page. 31 findings total: 5 blockers, 10 major, 11 minor, 5 info.

### Blockers — Addressed

| Finding | Resolution |
|---|---|
| No formal mapping from access control to Schubert geometry | ✅ [Distributed Game Sync](./distributed-game-sync.md) — formal derivation from game state to Grassmannian |
| "Configuration" count never operationally defined | ✅ Configuration = valid game state reconciliation (game sync §4) |
| Impossibility detection claim unsubstantiated | ✅ σ₂·σ₁₁ = 0 = combat mode + safe zone (game sync §3). Boolean AND false positive documented. |
| Algebraic identities mislabeled as security properties | ✅ Relabeled "algebraic identities" with caveat that security relevance depends on domain mapping |
| Scaling unaddressed (#P-hardness) | ✅ Game sync §6: complexity acknowledged, practical bounds given (10K players × Gr(2,4) = 2ms) |

### Major — Addressed

| Finding | Resolution |
|---|---|
| No threat model | ✅ [Threat Model](./threat-model.md) — adversary capabilities, security properties |
| Missing flag structure explanation | ✅ [Flag Structure](./flag-structure.md) — reference flag = clearance hierarchy |
| No Rust code in introduction | ✅ Quick start code block added |
| Fragile dependencies (pre-1.0 siblings) | ✅ Dependency table clarifies optional vs required |

### Minor/Info — Addressed

| Finding | Resolution |
|---|---|
| Notation issues (σ vs partition, "point class") | ✅ Notation guide added to concepts |
| Doc structure (What's New interrupts math) | ✅ Moved to end of README |
| Adversarial concerns (DoS, model injection) | ✅ [Adversarial Concerns](./adversarial-concerns.md) |
| Non-identifiability (who has access?) | ✅ Capacity vs occupancy explanation in threat model |
| Audit trail (commutativity eliminates temporal) | ✅ Semantic commutativity ≠ temporal audit discussed |
| Missing prior art references | ✅ References added to concepts page |

### Deferred to arXiv Preprint or Future Release

| Finding | Why Deferred |
|---|---|
| Empirical comparison to Cedar/OPA/Casbin | Needs benchmark implementation against real policy engines |
| Trust calibration (formal model) | Formal mathematical work — belongs in the paper |
| Cross-domain intersection derivation | Needs flag variety embedding proof — paper material |
| Rate limiting geometric justification | Needs empirical data connecting intersection numbers to throughput |
| Verification artifacts (proofs, certificates) | Depends on Karpal proof publication pipeline |
| Composition associativity proof | Formal category-theoretic proof — paper material |

## Ongoing Challenges

### Learning Curve
Schubert calculus is not standard security engineering knowledge. We recommend:
- Start with [Distributed Game Sync](./distributed-game-sync.md) — the most
  concrete motivating example
- Start with Gr(2,4) — the standard RBAC space
- Use the [Getting Started](../getting-started.md) walkthrough
- Use `schubert discover` to explore the API surface
- Read [Mathematical Foundation](../concepts/math.md) for the geometry

### Persistence
No built-in storage layer. All state is in-memory. Use `serde` serialization +
your database of choice for persistence. Future work: SQLite/PostgreSQL backends.

### Real-World Adoption
Schubert is a new project. We welcome:
- Production deployment reports
- Bug reports and edge cases
- Integration examples with common stacks (PostgreSQL, Redis, Kubernetes)

## Future Directions

See the full [Roadmap](./roadmap.md) for speculative directions including:
- Persistent backends (SQLite, PostgreSQL)
- gRPC policy distribution protocol
- Policy diff and incremental updates
- Visualization of Schubert varieties
- Integration with OpenFGA / Rego policy languages
- Empirical comparison to Cedar, OPA, Casbin, Oso
- Formal trust calibration model
