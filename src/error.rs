// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: AGPL-3.0-only

//! Error types for Schubert access control.
//!
//! All fallible operations return [`Result<T>`], which is an alias for
//! `std::result::Result<T, SchubertError>`. Errors are structured with
//! context-rich variants using `thiserror`.

use amari_enumerative::EnumerativeError;
use thiserror::Error;

/// Errors that can occur in Schubert access control operations.
///
/// Each variant carries contextual information: capability IDs,
/// principal IDs, partition details, and Grassmannian parameters.
#[derive(Error, Debug)]
pub enum SchubertError {
    /// A capability with this ID is already registered.
    #[error("capability '{0}' already exists")]
    CapabilityExists(String),

    /// The requested capability ID was not found in the registry.
    #[error("capability '{0}' not found")]
    CapabilityNotFound(String),

    /// A principal with this ID already exists in the controller.
    #[error("principal '{0}' already exists")]
    PrincipalExists(String),

    /// The requested principal was not found.
    #[error("principal '{0}' not found")]
    PrincipalNotFound(String),

    /// The principal already holds this capability.
    #[error("principal '{principal}' already holds capability '{capability}'")]
    CapabilityAlreadyGranted {
        /// The principal attempting to be granted.
        principal: String,
        /// The capability they already possess.
        capability: String,
    },

    /// The principal does not hold this capability.
    #[error("principal '{principal}' does not hold capability '{capability}'")]
    CapabilityNotHeld {
        /// The principal being checked.
        principal: String,
        /// The capability they do not hold.
        capability: String,
    },

    /// The partition is not valid for the current Grassmannian.
    #[error("invalid partition {partition:?} for Gr({k},{n}): {reason}")]
    InvalidPartition {
        /// The partition that was rejected.
        partition: Vec<usize>,
        /// Grassmannian parameter k.
        k: usize,
        /// Grassmannian parameter n.
        n: usize,
        /// Why the partition was rejected.
        reason: String,
    },

    /// The Grassmannian parameters k,n are invalid.
    #[error("invalid Grassmannian Gr({k},{n}): {reason}")]
    InvalidGrassmannian {
        /// Grassmannian parameter k.
        k: usize,
        /// Grassmannian parameter n.
        n: usize,
        /// Why the Grassmannian is invalid.
        reason: String,
    },

    /// An error from the underlying amari enumerative engine.
    #[error("enumerative error: {0}")]
    Enumerative(#[from] EnumerativeError),

    /// Composition of principals failed.
    #[error("composition failed: {0}")]
    CompositionFailed(String),

    /// The access controller has no configured Grassmannian.
    #[error("access controller has no configured Grassmannian")]
    NoGrassmannianConfigured,

    /// I/O error from an audit sink.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization error from an audit sink.
    #[error("serialization error: {0}")]
    Serialization(String),

    /// Policy parse error.
    #[error("policy parse error: {0}")]
    PolicyParseError(String),

    /// Policy export error.
    #[error("policy export error: {0}")]
    PolicyExportError(String),

    /// Generic error for multi-controller and other operations.
    #[error("{0}")]
    Generic(String),

    /// Cryptographic verification failed.
    #[error("crypto verification failed: {0}")]
    CryptoVerificationFailed(String),

    /// Rate limit exceeded for a principal.
    #[error("rate limit exceeded for principal '{principal}': {available:.2} tokens available out of {capacity:.2}")]
    RateLimitExceeded {
        /// The principal that exceeded the rate limit.
        principal: String,
        /// Tokens available at time of check.
        available: f64,
        /// Maximum token capacity.
        capacity: f64,
    },
}

/// Result type alias for Schubert operations.
pub type Result<T> = std::result::Result<T, SchubertError>;
