// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: AGPL-3.0-only

//! Surreal trust levels for Schubert access control.
//!
//! Replaces `f64` trust with exact surreal arithmetic from Amari 0.23.0.
//! Built on `RationalSurreal` for general rational trust and
//! `EpsilonPolynomial` for infinitesimal trust levels.
//! See `docs/surreal-trust-levels.md` for the full expansion.

use amari_surreal::epsilon::EpsilonPolynomial;
use amari_surreal::rational::RationalSurreal;
use num_traits::ToPrimitive;
use std::cmp::Ordering;

/// A trust level backed by exact surreal arithmetic (Amari 0.23.0).
#[derive(Debug, Clone)]
pub struct SurrealTrust {
    finite: RationalSurreal,
    infinitesimal: Option<EpsilonPolynomial>,
}

impl SurrealTrust {
    /// Full trust (1.0).
    pub fn full() -> Self {
        Self {
            finite: RationalSurreal::one(),
            infinitesimal: None,
        }
    }

    /// No trust (0.0).
    pub fn none() -> Self {
        Self::zero()
    }

    /// Zero trust.
    pub fn zero() -> Self {
        Self {
            finite: RationalSurreal::zero(),
            infinitesimal: None,
        }
    }

    /// Create from a rational p/q.
    pub fn from_ratio(numer: i64, denom: u64) -> Self {
        let n = num_bigint::BigInt::from(numer);
        let d = num_bigint::BigInt::from(denom);
        Self {
            finite: RationalSurreal::from_ratio(n, d).expect("valid rational"),
            infinitesimal: None,
        }
    }

    /// Create from an integer.
    pub fn from_integer(n: i64) -> Self {
        Self {
            finite: RationalSurreal::from_integer(n),
            infinitesimal: None,
        }
    }

    /// Create from an f64 (lossy).
    pub fn from_f64(value: f64) -> Self {
        if value == 0.0 || !value.is_finite() {
            return Self::zero();
        }
        let bits = value.to_bits();
        let sign = if (bits >> 63) != 0 { -1i64 } else { 1i64 };
        let exponent = ((bits >> 52) & 0x7ff) as i32 - 1023;
        let mantissa_bits = bits & 0x000f_ffff_ffff_ffff;
        let mantissa = if exponent > -1023 {
            (mantissa_bits | 0x0010_0000_0000_0000) as i64
        } else {
            mantissa_bits as i64
        };
        let adj = if exponent > -1023 {
            exponent - 52
        } else {
            -1022 - 52
        };
        let numerator = sign * mantissa;
        if adj >= 0 {
            Self::from_integer(numerator << (adj as u32))
        } else {
            Self::from_ratio(numerator, 1u64 << ((-adj) as u32))
        }
    }

    /// Approximate as f64.
    pub fn approximate(&self) -> f64 {
        let n = self.finite.numer().to_f64().unwrap_or(0.0);
        let d = self.finite.denom().to_f64().unwrap_or(1.0);
        n / d
    }

    /// Create the positive infinitesimal ε.
    pub fn epsilon() -> Self {
        Self {
            finite: RationalSurreal::zero(),
            infinitesimal: Some(EpsilonPolynomial::epsilon()),
        }
    }

    /// Create ε^n.
    pub fn epsilon_power(n: i32) -> Self {
        Self {
            finite: RationalSurreal::zero(),
            infinitesimal: Some(EpsilonPolynomial::monomial(RationalSurreal::one(), n)),
        }
    }

    /// Check for infinitesimal component.
    pub fn has_infinitesimal(&self) -> bool {
        self.infinitesimal
            .as_ref()
            .is_some_and(|eps| !eps.is_zero())
    }

    /// Check if purely finite.
    pub fn is_purely_finite(&self) -> bool {
        !self.has_infinitesimal()
    }

    /// Clamp to [0.0, 1.0].
    pub fn clamp_unit(&self) -> Self {
        let zero = RationalSurreal::zero();
        let one = RationalSurreal::one();
        if self.finite < zero {
            Self {
                finite: zero,
                infinitesimal: None,
            }
        } else if self.finite > one {
            Self {
                finite: one,
                infinitesimal: None,
            }
        } else {
            self.clone()
        }
    }
}

impl PartialEq for SurrealTrust {
    fn eq(&self, other: &Self) -> bool {
        self.finite == other.finite && self.infinitesimal == other.infinitesimal
    }
}

impl Eq for SurrealTrust {}

impl PartialOrd for SurrealTrust {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SurrealTrust {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.finite.cmp(&other.finite) {
            Ordering::Equal => compare_infinitesimal(&self.infinitesimal, &other.infinitesimal),
            ord => ord,
        }
    }
}

/// Compare two optional epsilon polynomials.
fn compare_infinitesimal(a: &Option<EpsilonPolynomial>, b: &Option<EpsilonPolynomial>) -> Ordering {
    match (a, b) {
        (None, None) => Ordering::Equal,
        (Some(eps), None) => {
            if eps.is_zero() {
                Ordering::Equal
            } else {
                Ordering::Greater
            }
        }
        (None, Some(eps)) => {
            if eps.is_zero() {
                Ordering::Equal
            } else {
                Ordering::Less
            }
        }
        (Some(a), Some(b)) => {
            if a.is_zero() && b.is_zero() {
                return Ordering::Equal;
            }
            // Compare by valuation (smaller exponent = larger value)
            let a_val = a.valuation().unwrap_or(0);
            let b_val = b.valuation().unwrap_or(0);
            match a_val.cmp(&b_val).reverse() {
                Ordering::Equal => {
                    // Same valuation — compare leading coefficient
                    let a_coeff = a
                        .terms()
                        .first_key_value()
                        .map(|(_, c)| c.clone())
                        .unwrap_or_else(RationalSurreal::zero);
                    let b_coeff = b
                        .terms()
                        .first_key_value()
                        .map(|(_, c)| c.clone())
                        .unwrap_or_else(RationalSurreal::zero);
                    a_coeff.cmp(&b_coeff)
                }
                ord => ord,
            }
        }
    }
}

/// Conversion from standard TrustLevel to SurrealTrust.
pub trait IntoSurealTrust {
    /// Convert this trust level to a surreal trust value.
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
        assert!((SurrealTrust::full().approximate() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn none_is_zero() {
        assert!((SurrealTrust::none().approximate()).abs() < 1e-10);
    }

    #[test]
    fn rational_ordering() {
        assert!(SurrealTrust::from_ratio(1, 3) < SurrealTrust::from_ratio(1, 2));
    }

    #[test]
    fn epsilon_positive() {
        let eps = SurrealTrust::epsilon();
        assert!(eps > SurrealTrust::zero());
        assert!(eps < SurrealTrust::from_ratio(1, 1_000_000));
    }

    #[test]
    fn epsilon_hierarchy() {
        // ε > ε² > 0
        assert!(SurrealTrust::epsilon() > SurrealTrust::epsilon_power(2));
        assert!(SurrealTrust::epsilon_power(2) > SurrealTrust::zero());
    }

    #[test]
    fn from_f64_roundtrip() {
        let t = SurrealTrust::from_f64(0.75);
        assert!((t.approximate() - 0.75).abs() < 1e-10);
    }

    #[test]
    fn trust_level_conversion() {
        let surreal = crate::TrustLevel::new(0.5).to_surreal();
        assert!((surreal.approximate() - 0.5).abs() < 1e-10);
    }

    #[test]
    fn epsilon_detection() {
        assert!(SurrealTrust::epsilon().has_infinitesimal());
        assert!(!SurrealTrust::full().has_infinitesimal());
    }
}
