# Rate Limiting

Schubert's `RateLimiter` uses a token-bucket algorithm scaled by Schubert intersection
numbers — higher-trust principals get proportionally more throughput.

## Basic Rate Limiter

```rust
use schubert::RateLimiter;

// 10 tokens per second, burst capacity of 20 tokens
let mut rl = RateLimiter::new(10.0, 1.0);

// Per-request rate check
if rl.try_consume("alice").is_err() {
    return Err("rate limit exceeded");
}
```

## Configure from Access Decision

```rust
let granted = acl.check(&alice, &["read", "write"])?;
let mut rl = RateLimiter::new(10.0, 1.0);

// Scale the rate limiter based on access decision:
// Higher intersection numbers → more tokens
rl.configure_from_decision("alice", &granted)?;
```

## Bucket State Queries

```rust
let available = rl.tokens_available("alice");
let capacity = rl.capacity("alice");
let fill_rate = rl.refill_rate();
```

## How It Works

The token bucket is parameterized by the Schubert intersection number from the
access decision. A principal with more valid configurations (higher intersection
number) gets a larger token bucket — this models the principle that higher-trust,
multi-capability access should have proportionally more throughput.

When trust degrades and the intersection number drops, the rate limit tightens
automatically.
