# Holographic Memory (Minuet)

Cosine-similarity-based access patterns via Minuet (v0.3). Enable the `holographic`
feature.

## HolographicAccessControl

```rust
use schubert::holographic::HolographicAccessControl;

let mut holo = HolographicAccessControl::new();

// Encode a principal's access pattern
holo.encode("alice", &["read", "write"])?;
holo.encode("bob", &["read"])?;

// Query by similarity
let similar = holo.query_similar(&["read", "write"], 5)?;
// Returns principals with similar access patterns, ranked by cosine similarity
```

## How It Works

1. Access patterns are encoded as vectors via FNV hash
2. Cosine similarity measures how close two access patterns are
3. Schubert intersection provides geometric validation of similarity
4. Results are ranked by combined similarity + intersection score

## Use Cases

- **Anomaly detection**: Flag access patterns unlike any known principal
- **Role discovery**: Cluster principals by access pattern similarity
- **Privilege escalation detection**: Sudden change in access pattern vector
- **Audit forensics**: Find principals with similar access to a known attacker

## Query

```rust
// Find top-K similar principals
let results = holo.query_similar(&["read:data"], 10)?;

for (principal, similarity) in results {
    println!("{principal}: {similarity:.4}");
}
```

## Limitations

- Cosine similarity is approximate, not exact match
- Encoding uses FNV hash (fast but not cryptographic)
- Not a full Minuet algebra binding — simplified for access control
- Memory-only storage (no persistence)
