# Critique of the Schubert Library

> **v0.1.0 Snapshot** — This document reflects the state of the project at its
> initial public release. Several points raised here have since been addressed:
> a [CLI](docs/guide/cli.md) now exists, the project is [dual-licensed](LICENSE-COMMERCIAL),
> and [comprehensive documentation](docs/guide/) has been written. We keep this
> critique public as an honest record of the project's starting point and a
> benchmark for future improvement.

## Overview
Schubert is a Rust library that replaces binary allow/deny access‑control with a *quantitative* model based on Schubert calculus.  While the concept is mathematically elegant and the implementation is solid, several practical concerns limit its suitability for mainstream commercial use.

---

## Key Criticisms

1. **License – AGPL‑3.0‑only**
   * The AGPL is a strong copyleft license.  Any downstream product that ships Schubert (or a derivative) must also be released under the AGPL, which is often unacceptable for commercial or proprietary software.  The repository does not offer a commercial‑license exception, so organizations must either open‑source their entire stack or abandon the library.

2. **Steep Learning Curve**
   * Understanding and writing policies requires familiarity with Schubert calculus, Grassmannians, and intersection theory.  This mathematical background is far beyond the typical skill set of security engineers or DevOps teams, creating a barrier to adoption.

3. **Sparse Ecosystem & Tooling**
   * The crate provides a TOML‑based policy DSL (`policy` feature) but no accompanying CLI, editor integration, or schema validation tools.  Users must build their own tooling to author, validate, and deploy policies.

4. **Performance Benchmarks Missing**
   * The documentation claims “fast checks” and “zero runtime dependencies”, yet there are no benchmark results or profiling data.  Without empirical evidence, it is hard to gauge suitability for high‑throughput services.

5. **No Built‑in Persistence**
   * Policies, audit logs, and capability state are stored only in memory (or via optional `serde`).  Production systems typically need durable storage (SQL, NoSQL, etc.), which must be added manually.

6. **Feature‑Flag Complexity**
   * The library ships many optional features (`serde`, `karpal`, `crypto`, `wasm`, `parallel`, `policy`).  Enabling the right combination can be confusing, and mismatched versions of optional dependencies may cause compile‑time conflicts.

7. **Documentation Depth**
   * High‑level README is good, but per‑module documentation is thin.  Developers often need to read source code to understand the API, increasing onboarding time.

8. **Limited Real‑World Adoption**
   * Schubert appears to be a research prototype with no reported production deployments.  This lack of community validation means bugs, edge‑case handling, and long‑term maintenance are unproven.

---

## Who Might Still Benefit
- **Academic or research projects** exploring quantitative access control.
- **High‑assurance systems** that need formal verification (via the optional `karpal` module).
- **Embedded/`_std` environments** where a tiny, dependency‑free access‑control core is required.

---

## Recommendations for Improvement
1. Offer a **dual‑license** model (e.g., AGPL + commercial) to broaden adoption.
2. Provide **tutorials** and **example policy files** that walk users through the mathematics.
3. Add a **CLI** for policy validation and generation.
4. Publish **benchmark results** for typical workloads (single check, batch check, WASM).
5. Implement a **persistent backend** (e.g., optional `serde` + file/DB storage).
6. Consolidate feature flags or provide a **feature‑selection guide**.
7. Expand **module‑level documentation** with usage examples and diagrams.

---

*Prepared by the coding‑agent on 2026‑06‑03.*