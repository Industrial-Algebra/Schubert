# Stability Analysis

Wall-crossing analysis of capability stability under trust degradation.

## analyze_stability()

```rust
use schubert::analyze_stability;

let report = analyze_stability(&acl, &principal)?;
```

## StabilityReport

```rust
pub struct StabilityReport {
    /// Breakpoints where stability changes
    pub phase_diagram: Vec<(f64, usize)>,
    /// Individual stability walls per capability
    pub walls: Vec<StabilityWall>,
    /// Which capability degrades first
    pub most_sensitive: String,
    /// Current stability at trust = 1.0
    pub at_full_trust: usize,
    /// Current stability at trust = 0.0
    pub at_zero_trust: usize,
}
```

## StabilityWall

```rust
pub struct StabilityWall {
    pub capability: String,
    pub cap_kind: CapabilityKind,
    /// Trust level where this capability crosses its stability wall
    pub trust_threshold: f64,
}
```

## How It Works

1. For each granted capability, compute its stability as a function of trust
2. Higher-codimension capabilities cross stability walls at higher trust levels
3. AdminLike capabilities cross first, ReadLike last
4. The phase diagram shows total viable configurations at each trust level

## Batch Stability (parallel feature)

```rust
let principals = vec![alice, bob, carol];
let reports = analyze_stability_batch(&acl, &principals)?;
```

## Composed Stability (v0.5.0)

`analyze_composed_stability` probes Roadmap #17's open question — *does the
composed phase diagram `P_C` derive from `P_A` and `P_B`?* — empirically. For
an operadic composition `A ∘_S B` (see [`composition`](./composition.md)), it
computes all three diagrams and tests `P_C` against the **deduplicated
additive baseline**: the stable-count of the *union* of the constituents'
retained capabilities. Where the baseline holds (`is_additive`), composition
is predictable; where it deviates, the deviating trust levels
(`non_additive_breakpoints`) are emergence signatures — the BPS-bound-state
analogy.

```rust
use schubert::stability::analyze_composed_stability;

let report = analyze_composed_stability(&acl, &alice, "handoff", &bob, "handoff")?;
assert!(report.is_additive); // under the current engine — the measured baseline
```

Under the current `WallCrossingEngine` the measured result is **additive** —
walls are per-capability — which this instrument establishes as the baseline
any interaction-aware engine must beat. Run
`cargo run --example wall_crossing_probe` to sweep a family of compositions
and see the verdicts; the docs (`docs/design/wall-crossing-diffusion-composition.md`)
frame the research direction this instrument serves.
