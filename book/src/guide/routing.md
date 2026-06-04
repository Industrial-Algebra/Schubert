# Schubert Routing

Geometric network routing — paths through Schubert conditions.

## RouteTable

```rust
use schubert::RouteTable;

let mut table = RouteTable::new(2, 4);

// Advertise a route as a Schubert condition
table.advertise("service-a", vec![1])?;
table.advertise("service-b", vec![2])?;

// Find a path through the route table
let path = table.find_path(&["service-a", "service-b"])?;
```

## Route Advertisements

Each route advertisement is a Schubert condition with a partition. Routes are
compatible if their Schubert intersection is non-empty:

```rust
// Advertise with different codimensions
table.advertise("gateway", vec![1])?;     // σ₁ — lightweight route
table.advertise("database", vec![2, 1])?;  // σ₂₁ — restricted route

// Routes compose if intersection > 0
let route = table.find_path(&["gateway", "database"])?;
```

## Congestion Detection

If the intersection number drops (fewer valid paths), the route table detects
congestion:

```rust
if let Some(congested) = table.check_congestion(&["gateway", "database"])? {
    println!("Route congested: {congested:?}");
}
```
