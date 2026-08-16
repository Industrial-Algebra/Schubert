# Composition Engine

Operadic composition of principals through shared capabilities.

## compose()

```rust
use schubert::compose;

let result = compose(&acl, &producer, "output", &consumer, "input")?;
```

## CompositionResult

```rust
pub enum CompositionResult {
    Composable { multiplicity: usize },
    NotComposable { reason: String },
}
```

- **Composable**: `multiplicity` configurations survive the composition
- **NotComposable**: Geometrically incompatible capabilities

## are_composable()

Cheaper pre-check before full composition:

```rust
if are_composable(&acl, "read", "write")? {
    let result = compose(&acl, &alice, "read", &bob, "write")?;
}
```

## Use Cases

- **Service chaining**: Service A produces output that Service B consumes
- **Delegation**: Principal delegates a capability to another principal
- **Cross-domain translation**: Translate capabilities between Grassmannians
- **Composability checking**: Verify two services can interoperate

## Properties

- **Commutative**: Order of composition doesn't affect result
- **Associative**: `(a ∘ b) ∘ c = a ∘ (b ∘ c)`
- **Zero-preserving**: If either side is impossible, composition is impossible
