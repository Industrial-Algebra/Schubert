# Roadmap & History

> **v0.1.0 Snapshot** — All 14 core roadmap items are complete. Speculative
> directions are explorations for the research community, not commitments.

## v0.1.0 — Foundation Complete

### Core Infrastructure
- AccessController with principal management, capability registry, grant/revoke
- Quantitative AccessDecision (Granted{n}, Impossible, Denied, Underconstrained)
- 4 computation paths: LR, Localization, Tropical, Matroid
- Operadic composition, stability analysis, pluggable audit sinks

### Feature-Gated Modules
- `serde` — Serialization, JSON I/O, roundtrip
- `policy` — TOML policy language with validation
- `wasm` — WasmController with JS bindings
- `crypto` — Ed25519 capability tokens
- `karpal` — Type-level proofs (Proven, Rewrite, law checks)
- `karpal-verify` — SMT/Lean proof obligations, Certified trust boundary
- `surreal` — RationalSurreal + EpsilonPolynomial trust arithmetic
- `holographic` — Minuet cosine-similarity access patterns

### Advanced Features
- Context-aware decisions (resource scoping, time-aware trust)
- MultiController with cross-domain capability translation
- Temporal access control (expiry, time-remaining)
- Rate limiting scaled by intersection numbers
- Schubert routing with geometric path computation
- Distributed CRDTs with version vectors

### Quality
- 128 unit tests + 18 CLI tests = 146 total
- Zero clippy warnings (all feature combinations)
- 7 example programs
- CI/CD: fmt, clippy, test matrix (5 combos), docs, wasm build, verification

## Speculative Directions

These are research explorations, not commitments:

1. **Persistent backends** — SQLite, PostgreSQL, Redis storage layers
2. **gRPC policy distribution** — Wire protocol for multi-node policy sync
3. **Policy diff engine** — Incremental policy updates with minimal recomputation
4. **Visualization** — SVG/WebGL rendering of Schubert varieties
5. **OpenFGA/Rego bridge** — Translation between Schubert policies and standard DSLs
6. **Holographic persistence** — Full Minuet store integration with cosine indexing
7. **Async runtime** — tokio-based async AccessController
8. **Policy fuzzing** — Automated discovery of impossible capability combinations
9. **Benchmark suite** — Standardized workloads with published results
10. **WASM Component Model** — WIT-based interface definitions
