// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: AGPL-3.0-only

//! Re-exports of `amari_enumerative` phantom types for compile-time verification.
//!
//! These zero-sized types encode mathematical properties at the type level.
//! Combined with [`karpal_proof::Proven`], they enable compile-time verification
//! of Schubert access control invariants.
//!
//! # Phantom Type Categories
//!
//! | Category | Marker Trait | States |
//! |----------|-------------|--------|
//! | Partition validity | [`PartitionValidity`] | [`ValidPartition`], [`UnvalidatedPartition`] |
//! | Grassmannian containment | [`BoxContainment`] | [`FitsInBox`], [`UnverifiedBox`] |
//! | Capability grant lifecycle | [`GrantState`] | [`Granted`], [`Pending`], [`Revoked`] |
//! | Intersection dimension | [`IntersectionDimension`] | [`Transverse`], [`Excess`], [`Deficient`] |
//!
//! # Composite Aliases (from amari)
//!
//! - [`ValidSchubertClass`] = `(ValidPartition, FitsInBox)` — fully validated capability
//! - [`ValidLRTableau`] = `(Semistandard, LatticeWord)` — valid LR tableau
//!
//! # Usage
//!
//! ```
//! use schubert::phantom::{Properties, ValidSchubertClass};
//!
//! // Zero-cost marker — no runtime overhead
//! let validated: Properties<ValidSchubertClass> = Properties::new();
//! assert_eq!(std::mem::size_of_val(&validated), 0);
//! ```

pub use amari_enumerative::phantom::{
    // Grassmannian containment
    BoxContainment,
    // Intersection dimension
    Deficient,
    Excess,
    FitsInBox,
    GrantState,
    // Capability grant states
    Granted,
    IntersectionDimension,
    // Tableau properties
    LatticeWord,
    // Partition validity
    PartitionValidity,
    Pending,
    // Generic wrapper
    Properties,
    Revoked,
    Semistandard,
    TableauValidity,
    Transverse,
    UnknownDimension,
    UnvalidatedPartition,
    UnverifiedBox,
    UnverifiedTableau,
    ValidLRTableau,
    ValidPartition,
    // Composite
    ValidSchubertClass,
};

/// Phantom markers specific to Schubert access control.
///
/// These type aliases combine amari phantom types for common
/// Schubert access control patterns.
pub mod markers {
    use super::*;

    /// A capability that has been registered and is ready for granting.
    ///
    /// Combines `ValidSchubertClass` (partition is valid and fits in the
    /// Grassmannian box) with the initial un-granted state.
    pub type RegisteredCapability = ValidSchubertClass;

    /// An access check that produced a finite result (transverse intersection).
    ///
    /// Both the intersection is transverse AND the result is finite (>0).
    pub type FiniteAccess = Transverse;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_phantom_types_are_zero_sized() {
        use std::mem::size_of;
        assert_eq!(size_of::<ValidPartition>(), 0);
        assert_eq!(size_of::<Granted>(), 0);
        assert_eq!(size_of::<Revoked>(), 0);
        assert_eq!(size_of::<Transverse>(), 0);
        assert_eq!(size_of::<FitsInBox>(), 0);
        assert_eq!(size_of::<Properties<ValidSchubertClass>>(), 0);
    }

    #[test]
    fn properties_is_copy() {
        let p1 = Properties::<ValidSchubertClass>::new();
        let _p2 = p1;
        assert_eq!(p1, Properties::new());
    }
}
