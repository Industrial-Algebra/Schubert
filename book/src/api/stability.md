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
