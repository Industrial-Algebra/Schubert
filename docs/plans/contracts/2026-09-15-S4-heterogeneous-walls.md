# Contract S4 — Heterogeneous Walls Probe (Schubert in-crate)

**Provenance:** Thatch founding doc `docs/ideation-edge-geometry.md` probe S4;
v0.6.0 sprint plan Workstream B. **Dispatched to:** deepseek moment.
**Branch:** `feature/s4-heterogeneous-walls` (already created; do NOT run any
git commands). **Work in:** `/home/elliotthall/working/industrial-algebra/Schubert`.

## Mission

Build the first instrument that crosses a **time wall** (grant expiry,
crypto-space) with a **trust wall** (partition stability, controller-space) in
one composition harness. Extend the #50 baseline ("all compositions additive —
walls are per-capability") from the pure-trust axis to the trust×time product.

**The seam under test:** the expiring capability `temp_sign` shares partition
λ=[2] with the permanent capability `audit`. Under per-capability walls, the
λ=[2] wall position in the composed diagram must survive `temp_sign`'s death
(`audit` holds it). If the breakpoint set changes beyond the predictable
capability-removal, that is heterogeneous emergence — a *finding*, not a bug.

**Semantic bridge (ADR-0001):** expired grant ⇒ capability dead ⇒ principal
rebuilt without it. `verify_at` instruments the time wall; a second controller
rebuild instruments the simulated revocation. No engine or API changes.

## Scenario (fixed)

- `AccessController::new(2, 4)` — Gr(2,4).
- Capabilities (register in this order):
  | id | partition | kind |
  |---|---|---|
  | `read` | `[1]` | `CapabilityKind::ReadLike` |
  | `temp_sign` | `[2]` | `CapabilityKind::WriteLike` |
  | `audit` | `[2]` | `CapabilityKind::ReadLike` |
  | `manage` | `[2, 1]` | `CapabilityKind::WriteLike` |
- Principal `wall-a` (`PrincipalId::new("wall-a")?`): pre-cliff holds
  `read`, `temp_sign`, `audit`; post-cliff holds `read`, `audit`.
- Principal `wall-b`: holds `read`, `manage` (both slices).
- Interface both sides: `"read"`.
- Grant: `issuer.issue_grant_with_expiry(pa, &[(CapabilityId::new("temp_sign")?, vec![2])], 1000)?`
- Time axis: `now = 999` (valid), `now = 1000` (expired — `verify_at` is
  inclusive: `now >= expires_at`).

## Files

### 1. NEW `examples/heterogeneous_walls_probe.rs`

Header mirrors `examples/wall_crossing_probe.rs` (license + module doc citing
"Thatch probe S4 — heterogeneous wall coordinates"). Body:

- `fn build_controller(with_temp_sign: bool) -> schubert::Result<AccessController>`
  — registers the four capabilities; creates `wall-a` (grants `read`,
  `temp_sign` iff `with_temp_sign`, `audit`) and `wall-b` (grants `read`,
  `manage`); returns the controller. Construction calls verbatim:
  `acl.register_capability(Capability::new(CapabilityId::new(id.clone()).expect("valid id"), label, partition, kind))?`
  (copy the pattern from `wall_crossing_probe.rs` lines 82–96).
- `fn breakpoint_values(report: &ComposedStabilityReport) -> Vec<f64>` —
  `report.phase_diagram_composed.phase_diagram` iterated, each
  `.trust_level.value()`, collected, sorted.
- `main()`:
  1. Issuer + verifier: `let issuer = CapabilityIssuer::generate();` /
     `let verifier = GrantVerifier::new(issuer.public_key())?;`
  2. Build pre-cliff controller + `pa`/`pb` ids, issue the grant (above),
     assert-with-println: `verify_at(&grant, 999)` → OK;
     `verify_at(&grant, 1000)` → matches `Err(SchubertError::GrantExpired { .. })`.
  3. `report_pre = analyze_composed_stability(&acl_pre, &pa, "read", &pb, "read")?`
  4. Build post-cliff controller (`build_controller(false)`), same ids,
     `report_post = ...` (re-issue nothing — no grant needed post-cliff).
  5. Print a two-slice table (mirror `wall_crossing_probe` formatting):
     per slice — composed capability count, `is_additive` verdict,
     breakpoint values; then the superposition verdict:
     `breakpoint_values(&report_pre) == breakpoint_values(&report_post)` →
     "WALLS SUPERPOSE … the λ=[2] wall survives temp_sign's death via audit"
     else "HETEROGENEOUS EMERGENCE DETECTED — report both breakpoint sets".
  6. Final RESULT block: honest, either way (mirror `wall_crossing_probe`
     lines 140–149 in tone; cite "trust×time product, current engine baseline").

### 2. NEW `tests/heterogeneous_walls_regression.rs`

First line: `#![cfg(feature = "crypto")]` + license header. Reuses the same
construction helpers (duplicate them in the test file; examples aren't
importable). Tests (names verbatim):

- `grant_valid_before_expiry` — `verify_at(&grant, 999)` is `Ok(())`.
- `grant_expires_inclusively` — `verify_at(&grant, 1000)` is
  `Err(SchubertError::GrantExpired { .. })`.
- `pre_cliff_composed_capability_count` — `report_pre.phase_diagram_composed.total_capabilities == 3`.
- `post_cliff_composed_capability_count` — `report_post.phase_diagram_composed.total_capabilities == 2`.
- `walls_superpose_across_time_cliff` — `breakpoint_values(&report_pre) == breakpoint_values(&report_post)` (sorted f64 vecs, exact equality — same engine, same partition sets).
- `ks_additivity_holds_in_both_slices` — `report_pre.is_additive && report_post.is_additive`.

**Pinned measurements:** the last two asserts encode the prediction. If
either fails, that is a genuine finding: STOP, report the actual values, do
NOT weaken or invert the assert. The verifier adjudicates.

### 3. EDIT `Cargo.toml`

Add:

```toml
[[example]]
name = "heterogeneous_walls_probe"
required-features = ["crypto"]
```

(mirror the existing `[[example]]` blocks' position/style).

## Constraints

- Zero new dependencies. Zero changes to `src/` (the probe is example +
  integration-test only — if you believe a `src/` change is required, STOP and
  report).
- No changes to any other example, test, book file, or workflow.
- TDD: write `tests/heterogeneous_walls_regression.rs` first, watch it fail to
  compile (no example/harness yet is fine — red), then implement.
- Style: license header on both new files
  (`// Copyright (C) 2026 Industrial Algebra` / `// SPDX-License-Identifier: Apache-2.0`).

## Completion (run all; report tail of each)

```bash
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo test --features crypto
cargo test --all-features
cargo run --example heterogeneous_walls_probe --features crypto 2>&1 | tail -30
```

Report: (a) each command's pass/fail line, (b) the example's full RESULT
block, (c) any ambiguity encountered — do not resolve ambiguities silently.
