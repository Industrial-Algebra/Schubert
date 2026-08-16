# Rate Limiter

Token-bucket rate limiting scaled by Schubert intersection numbers.

**Source**: `examples/rate_limiter.rs`

## Pattern

```rust
use schubert::RateLimiter;

let mut rl = RateLimiter::new(10.0, 1.0); // 10 tokens/sec

// Check access first
let granted = acl.check(&alice, &["read", "write"])?;

// Scale rate limiter by intersection number
rl.configure_from_decision("alice", &granted)?;

// Per-request check
for _ in 0..100 {
    if rl.try_consume("alice").is_err() {
        println!("Rate limit reached");
        break;
    }
}
```

## Key Takeaway

Higher-trust principals (higher intersection numbers) get proportionally more
throughput because the rate limiter is scaled by the access decision.
