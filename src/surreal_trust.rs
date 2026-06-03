// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: AGPL-3.0-only

//! Surreal trust levels for Schubert access control.
//!
//! Replaces `f64` trust with exact surreal arithmetic from `amari-surreal`.
//! Currently built on `Dyadic` (dyadic rationals m/2^n — finer than float).
//! Will upgrade to `RationalSurreal` and `EpsilonPolynomial` when Amari 0.23
//! merges. See `docs/surreal-trust-levels.md` for the full expansion.
//!
//! # Trust Level Lattice
//!
//! | Layer | Supported Now | Example |
//! |-------|--------------|---------|
//! | Finite real | ✅ via Dyadic | `SurrealTrust::from_f64(0.5)` |
//! | Dyadic rational | ✅ native | `SurrealTrust::dyadic(1, 2)` = 1/2 |
//! | General rational | 🔜 Amari 0.23 | `3/7` via RationalSurreal |
//! | Infinitesimal | 🔜 Amari 0.23 | ε, ε² via EpsilonPolynomial |
//! | Mixed | 🔜 Amari 0.23 | 0.5 + ε |
//!
//! # Example
//!
//! ```
//! # #[cfg(feature = "surreal")] {
//! use schubert::surreal_trust::SurrealTrust;
//!
//! let full = SurrealTrust::full();
//! let half: f64 = full.approximate();
//! assert!((half - 1.0).abs() < 1e-10);
//!
//! let small = SurrealTrust::dyadic(1, 1024); // 1/1024
//! assert!(small < full);
//! # }
//! ```

use amari_surreal::dyadic::Dyadic;
use num_traits::ToPrimitive;
use std::cmp::Ordering;

/// A trust level backed by exact surreal arithmetic.
///
/// Currently wraps `Dyadic` (dyadic rationals m/2^n), providing:
/// - Exact ordering (no floating-point rounding)
/// - Full arithmetic (add, sub, mul, div)
/// - Conversion to/from `f64` for backward compatibility
///
/// When Amari 0.23 merges, this will upgrade to `RationalSurreal` with
/// infinitesimal support via `EpsilonPolynomial`.
#[derive(Debug, Clone)]
pub struct SurrealTrust {
    value: Dyadic,
}

impl SurrealTrust {
    /// Full trust (1.0).
    pub fn full() -> Self {
        Self {
            value: Dyadic::new(1, 0),
        }
    }

    /// No trust (0.0).
    pub fn none() -> Self {
        Self {
            value: Dyadic::new(0, 0),
        }
    }

    /// Create from a dyadic rational m / 2^n.
    ///
    /// `SurrealTrust::dyadic(1, 2)` = 1/4 = 0.25.
    /// `SurrealTrust::dyadic(1, 1)` = 1/2 = 0.5.
    pub fn dyadic(mantissa: i64, exponent: u32) -> Self {
        Self {
            value: Dyadic::new(mantissa, exponent),
        }
    }

    /// Create from an f64 by converting to the nearest dyadic rational.
    ///
    /// Uses the IEEE 754 representation: f64 = mantissa * 2^exponent.
    pub fn from_f64(value: f64) -> Self {
        if value == 0.0 {
            return Self {
                value: Dyadic::zero(),
            };
        }
        if !value.is_finite() {
            return Self {
                value: Dyadic::zero(),
            };
        }

        let bits = value.to_bits();
        let sign = if (bits >> 63) != 0 { -1i64 } else { 1i64 };
        let exponent = ((bits >> 52) & 0x7ff) as i32 - 1023;
        let mantissa_bits = bits & 0x000f_ffff_ffff_ffff;

        // Normal numbers have implicit leading 1
        let mantissa = if exponent > -1023 {
            (mantissa_bits | 0x0010_0000_0000_0000) as i64
        } else {
            // Subnormal: no implicit leading 1, exponent is -1022
            mantissa_bits as i64
        };

        let adjusted_exponent = if exponent > -1023 {
            exponent - 52
        } else {
            -1022 - 52
        };

        let numerator = sign * mantissa;
        if adjusted_exponent >= 0 {
            // m * 2^e — shift left
            let value = numerator << (adjusted_exponent as u32);
            Self {
                value: Dyadic::from_integer::<i64>(value),
            }
        } else {
            // m / 2^|e| — use dyadic
            Self {
                value: Dyadic::new(numerator, (-adjusted_exponent) as u32),
            }
        }
    }

    /// Convert to an f64 approximation.
    pub fn approximate(&self) -> f64 {
        let numer = self.value.numer();
        let exp = self.value.exponent();
        let num_f64 = numer.to_f64().unwrap_or(0.0);
        if exp == 0 {
            num_f64
        } else {
            num_f64 / 2.0_f64.powi(exp as i32)
        }
    }

    /// Convert to inner Dyadic value.
    pub fn inner(&self) -> &Dyadic {
        &self.value
    }

    /// Clamp to [0.0, 1.0] range.
    pub fn clamp_unit(&self) -> Self {
        let zero = Dyadic::new(0, 0);
        let one = Dyadic::new(1, 0);
        if self.value < zero {
            Self { value: zero }
        } else if self.value > one {
            Self { value: one }
        } else {
            self.clone()
        }
    }
}

impl PartialEq for SurrealTrust {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl Eq for SurrealTrust {}

impl Ord for SurrealTrust {
    fn cmp(&self, other: &Self) -> Ordering {
        self.value.cmp(&other.value)
    }
}

impl PartialOrd for SurrealTrust {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Extension trait for `TrustLevel` to provide surreal conversion.
pub trait IntoSurealTrust {
    /// Convert a standard trust level to surreal trust.
    fn to_surreal(&self) -> SurrealTrust;
}

impl IntoSurealTrust for crate::TrustLevel {
    fn to_surreal(&self) -> SurrealTrust {
        SurrealTrust::from_f64(self.value())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_is_one() {
        let t = SurrealTrust::full();
        assert!((t.approximate() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn none_is_zero() {
        let t = SurrealTrust::none();
        assert!((t.approximate() - 0.0).abs() < 1e-10);
    }

    #[test]
    fn dyadic_ordering() {
        let quarter = SurrealTrust::dyadic(1, 2); // 1/4
        let half = SurrealTrust::dyadic(1, 1); // 1/2
        let three_quarters = SurrealTrust::dyadic(3, 2); // 3/4

        assert!(quarter < half);
        assert!(half < three_quarters);
        assert!(quarter < three_quarters);
        assert!(quarter < SurrealTrust::full());
    }

    #[test]
    fn dyadic_equality() {
        let a = SurrealTrust::dyadic(2, 1); // 2/2 = 1
        let b = SurrealTrust::dyadic(4, 2); // 4/4 = 1
        assert_eq!(a, b);
    }

    #[test]
    fn clamp_to_range() {
        let too_high = SurrealTrust::dyadic(3, 0); // 3 → clamp to 1
        assert!((too_high.clamp_unit().approximate() - 1.0).abs() < 1e-10);

        let too_low = SurrealTrust::dyadic(-1, 0); // -1 → clamp to 0
        assert!((too_low.clamp_unit().approximate() - 0.0).abs() < 1e-10);

        let ok = SurrealTrust::dyadic(1, 1); // 0.5 → unchanged
        assert!((ok.clamp_unit().approximate() - 0.5).abs() < 1e-10);
    }

    #[test]
    fn from_f64_roundtrip() {
        let t = SurrealTrust::from_f64(0.75);
        assert!((t.approximate() - 0.75).abs() < 1e-10);
    }

    #[test]
    fn trust_level_conversion() {
        let tl = crate::TrustLevel::new(0.5);
        let surreal = tl.to_surreal();
        assert!((surreal.approximate() - 0.5).abs() < 1e-10);
    }
}
