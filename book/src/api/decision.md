# Decision & Context

## AccessDecision

The quantitative result of an access check.

```rust
pub enum AccessDecision {
    Granted { configurations: usize },
    Impossible { conflicting: Vec<String> },
    Denied,
    Underconstrained { dimension: usize },
}
```

### Decision Logic

| Condition | Decision |
|---|---|
| Intersection number > 0 | `Granted { configurations }` |
| Intersection number = 0 | `Impossible { conflicting }` |
| Total codimension > Gr dimension | `Denied` |
| Total codimension < Gr dimension | `Underconstrained { dimension }` |

### Methods

```rust
impl AccessDecision {
    pub fn is_granted(&self) -> bool;
    pub fn is_impossible(&self) -> bool;
    pub fn is_denied(&self) -> bool;
    pub fn is_underconstrained(&self) -> bool;
    pub fn grant_count(&self) -> Option<usize>;
}
```

## ComputationPath

Four engines for computing Schubert intersections:

```rust
pub enum ComputationPath {
    LR,            // Default — balanced performance
    Localization,  // Geometric insight into *why*
    Tropical,      // Large-scale batch operations
    Matroid,       // Parallel evaluation
}
```

## AccessContext

Context-aware access with resource scoping and trust requirements:

```rust
pub struct AccessContext {
    /// Optional resource identifier for scoping
    pub resource: Option<String>,
    /// Time budget for the check (milliseconds)
    pub time_budget_ms: Option<u64>,
    /// Minimum trust level required (0.0–1.0)
    pub required_trust: f64,
}
```

Used with `check_with_context()` for time-aware, resource-scoped, trust-gated
access decisions.
