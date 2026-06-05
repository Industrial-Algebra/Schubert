// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: AGPL-3.0-only

//! Quantitative rate limiting via Schubert intersection numbers.
//!
//! Blends Schubert calculus with token-bucket rate limiting. The intersection
//! number from an access decision determines the bucket capacity — access with
//! 2 configurations gets 2× the rate of access with 1. The geometry of access
//! maps to the geometry of throughput.
//!
//! # How It Works
//!
//! 1. An access check produces an [`AccessDecision::Granted { configurations: n }`]
//! 2. The intersection number `n` becomes the base capacity multiplier
//! 3. A token bucket with capacity `n * base_capacity` is maintained per principal
//! 4. Each access consumes one token; tokens refill at a configurable rate
//!
//! # Example
//!
//! ```
//! use schubert::rate_limit::RateLimiter;
//!
//! let mut rl = RateLimiter::new(10.0, 1.0); // 10 tokens/sec base, 1.0 multiplier
//! rl.configure_principal("alice", 2);    // alice has 2 configs → 20 tokens/sec
//!
//! assert!(rl.try_consume("alice").is_ok());   // consumes 1 token
//! ```

use std::collections::HashMap;

use crate::{error::Result, AccessDecision, PrincipalId, SchubertError};

/// A rate limiter that uses Schubert intersection numbers for capacity.
///
/// Each principal gets a token bucket whose capacity is scaled by their
/// intersection number. A principal with σ₁⁴ = 2 configurations gets
/// twice the rate of a principal with only 1 configuration.
#[derive(Debug)]
pub struct RateLimiter {
    /// Number of tokens per second per configuration.
    base_rate: f64,
    /// Multiplier applied to intersection numbers.
    multiplier: f64,
    /// Per-principal bucket state.
    buckets: HashMap<PrincipalId, Bucket>,
}

/// A token bucket for a single principal.
#[derive(Debug, Clone)]
struct Bucket {
    tokens: f64,
    capacity: f64,
    refill_rate: f64, // tokens per second
    last_refill_ms: u64,
}

impl RateLimiter {
    /// Create a new rate limiter.
    ///
    /// - `base_tokens_per_second`: tokens granted per second per configuration.
    /// - `multiplier`: scales the intersection number. Use 1.0 for 1:1 mapping.
    pub fn new(base_tokens_per_second: f64, multiplier: f64) -> Self {
        Self {
            base_rate: base_tokens_per_second,
            multiplier,
            buckets: HashMap::new(),
        }
    }

    /// Configure a principal's rate limit based on an intersection number.
    ///
    /// Capacity = `intersection_number * multiplier * base_rate`.
    /// The bucket starts full.
    pub fn configure_principal(
        &mut self,
        principal_id: impl Into<PrincipalId>,
        intersection_number: u64,
    ) {
        let capacity = intersection_number as f64 * self.multiplier * self.base_rate;
        let bucket = Bucket {
            tokens: capacity,
            capacity,
            refill_rate: self.base_rate * intersection_number as f64 * self.multiplier,
            last_refill_ms: crate::principal::now_millis(),
        };
        self.buckets.insert(principal_id.into(), bucket);
    }

    /// Configure a principal from an access decision.
    ///
    /// Extracts the intersection number from `Granted` decisions.
    /// Returns an error if the decision is not `Granted`.
    pub fn configure_from_decision(
        &mut self,
        principal_id: impl Into<PrincipalId>,
        decision: &AccessDecision,
    ) -> Result<()> {
        match decision {
            AccessDecision::Granted { configurations, .. } => {
                self.configure_principal(principal_id, *configurations);
                Ok(())
            }
            other => Err(SchubertError::Generic(format!(
                "cannot configure rate limit from decision: {other:?}"
            ))),
        }
    }

    /// Attempt to consume one token for a principal.
    ///
    /// Returns `Ok(tokens_remaining)` if a token was available,
    /// `Err(RateLimitExceeded)` if the bucket is empty.
    pub fn try_consume(&mut self, principal_id: impl Into<PrincipalId>) -> Result<f64> {
        let pid = principal_id.into();
        let now = crate::principal::now_millis();

        let bucket = self.buckets.get_mut(&pid).ok_or_else(|| {
            SchubertError::Generic(format!(
                "principal '{pid}' not configured for rate limiting"
            ))
        })?;

        Self::refill(bucket, now);

        if bucket.tokens >= 1.0 {
            bucket.tokens -= 1.0;
            Ok(bucket.tokens)
        } else {
            Err(SchubertError::RateLimitExceeded {
                principal: pid.to_string(),
                available: bucket.tokens,
                capacity: bucket.capacity,
            })
        }
    }

    /// Check whether a principal can consume a token without actually consuming it.
    pub fn can_consume(&mut self, principal_id: impl Into<PrincipalId>) -> bool {
        let pid = principal_id.into();
        let now = crate::principal::now_millis();

        if let Some(bucket) = self.buckets.get_mut(&pid) {
            Self::refill(bucket, now);
            bucket.tokens >= 1.0
        } else {
            false
        }
    }

    /// Get the current token count for a principal (after refill).
    pub fn tokens_available(&mut self, principal_id: impl Into<PrincipalId>) -> Option<f64> {
        let pid = principal_id.into();
        let now = crate::principal::now_millis();

        self.buckets.get_mut(&pid).map(|bucket| {
            Self::refill(bucket, now);
            bucket.tokens
        })
    }

    /// Get the configured capacity for a principal.
    pub fn capacity(&self, principal_id: impl Into<PrincipalId>) -> Option<f64> {
        let pid = principal_id.into();
        self.buckets.get(&pid).map(|b| b.capacity)
    }

    /// Remove a principal from the rate limiter.
    pub fn remove_principal(&mut self, principal_id: &PrincipalId) {
        self.buckets.remove(principal_id);
    }

    /// Reset all buckets to full capacity.
    pub fn reset_all(&mut self) {
        let now = crate::principal::now_millis();
        for bucket in self.buckets.values_mut() {
            bucket.tokens = bucket.capacity;
            bucket.last_refill_ms = now;
        }
    }

    fn refill(bucket: &mut Bucket, now_ms: u64) {
        let elapsed_secs = now_ms.saturating_sub(bucket.last_refill_ms) as f64 / 1000.0;
        if elapsed_secs > 0.0 {
            bucket.tokens =
                (bucket.tokens + elapsed_secs * bucket.refill_rate).min(bucket.capacity);
            bucket.last_refill_ms = now_ms;
        }
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new(10.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn configure_and_consume() {
        let mut rl = RateLimiter::new(10.0, 1.0);
        rl.configure_principal("alice", 2);

        // Should have 2 * 1.0 * 10.0 = 20 tokens
        assert!(rl.try_consume("alice").is_ok());
        assert!(rl.try_consume("alice").is_ok());
    }

    #[test]
    fn exhaust_and_refill() {
        let mut rl = RateLimiter::new(1.0, 1.0); // 1 token per config per second
        rl.configure_principal("alice", 1); // 1 config → 1 token capacity

        // Consume the only token
        assert!(rl.try_consume("alice").is_ok());
        // Second consume should fail
        assert!(rl.try_consume("alice").is_err());
    }

    #[test]
    fn higher_intersection_gets_more() {
        let mut rl = RateLimiter::new(10.0, 1.0);
        rl.configure_principal("alice", 1); // 10 tokens
        rl.configure_principal("bob", 4); // 40 tokens

        let alice_cap = rl.capacity("alice").unwrap();
        let bob_cap = rl.capacity("bob").unwrap();

        // Bob should have 4x the capacity
        assert!((bob_cap / alice_cap - 4.0).abs() < 0.01);
    }

    #[test]
    fn configure_from_granted_decision() {
        let decision = AccessDecision::Granted {
            configurations: 3,
            path: crate::ComputationPath::LittlewoodRichardson,
        };
        let mut rl = RateLimiter::new(5.0, 1.0);
        rl.configure_from_decision("alice", &decision).unwrap();

        // 3 configs * 1.0 * 5.0 = 15 tokens
        assert!((rl.capacity("alice").unwrap() - 15.0).abs() < 0.01);
    }

    #[test]
    fn configure_from_denied_fails() {
        let mut rl = RateLimiter::new(5.0, 1.0);
        assert!(rl
            .configure_from_decision("alice", &AccessDecision::Denied)
            .is_err());
    }

    #[test]
    fn can_consume_check() {
        let mut rl = RateLimiter::new(0.1, 1.0); // 0.1 token/sec/config
        rl.configure_principal("alice", 1);
        // With 0.1 rate, barely any tokens — but at least 0 capacity means can_consume is false
        assert!(!rl.can_consume("alice"));
    }

    #[test]
    fn remove_and_missing_principal() {
        let mut rl = RateLimiter::new(10.0, 1.0);
        rl.configure_principal("alice", 1);

        let alice = PrincipalId::new("alice");
        rl.remove_principal(&alice);
        assert!(rl.capacity("alice").is_none());
        assert!(rl.try_consume("alice").is_err());
    }
}
