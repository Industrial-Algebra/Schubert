//! Audit trail for access control decisions.
//!
//! The audit module provides a pluggable recording mechanism for every
//! access decision made by the [`AccessController`](crate::AccessController).
//! Decisions are recorded as immutable [`DecisionRecord`]s and written
//! to an [`AuditSink`] — a trait that callers implement for their own
//! storage backend.
//!
//! # Design
//!
//! Audit is **asynchronous to the access decision** — a failing audit
//! sink never blocks or rejects an access check. The controller calls
//! `record()` after the decision is made, silently swallowing errors.
//! This ensures audit cannot become a denial-of-service vector.
//!
//! # Implementing a Sink
//!
//! ```rust
//! use schubert::audit::{AuditSink, DecisionRecord};
//! use std::sync::Mutex;
//!
//! struct JsonFileAudit {
//!     path: std::path::PathBuf,
//!     buffer: Mutex<Vec<DecisionRecord>>,
//! }
//!
//! impl AuditSink for JsonFileAudit {
//!     fn record(&self, record: &DecisionRecord) -> schubert::Result<()> {
//!         self.buffer.lock().unwrap().push(record.clone());
//!         Ok(())
//!     }
//! }
//! ```

use crate::capability::CapabilityId;
use crate::decision::AccessDecision;
use crate::error::Result;
use crate::principal::PrincipalId;

/// A single recorded access control decision.
///
/// Immutable once created. Forms an append-only audit trail when
/// collected by an [`AuditSink`].
#[derive(Debug, Clone)]
pub struct DecisionRecord {
    /// The principal who made the request.
    pub principal: PrincipalId,
    /// The capability IDs that were required.
    pub capabilities: Vec<CapabilityId>,
    /// The decision that was returned.
    pub decision: AccessDecision,
    /// Unix timestamp of the decision (milliseconds since epoch).
    pub timestamp: u64,
}

/// A sink for recording access control decisions.
///
/// Implementations write to files, databases, log streams, or any
/// other storage. The trait is synchronous by design — audit recording
/// should not introduce async complexity into the access check hot path.
pub trait AuditSink: Send + Sync {
    /// Record an access control decision.
    ///
    /// Errors are logged but never propagated to the access check —
    /// a failing audit sink does not block authorization.
    fn record(&self, record: &DecisionRecord) -> Result<()>;
}

/// An in-memory audit log for testing and small deployments.
///
/// Stores records in a `Vec<DecisionRecord>` behind a mutex. Not suitable
/// for production (not persistent, unbounded memory growth), but useful for
/// tests, examples, and development.
#[derive(Debug, Default)]
pub struct InMemoryAudit {
    records: std::sync::Mutex<Vec<DecisionRecord>>,
}

impl InMemoryAudit {
    /// Create a new empty audit log.
    pub fn new() -> Self {
        Self::default()
    }

    /// Return all recorded decisions in chronological order.
    pub fn records(&self) -> Vec<DecisionRecord> {
        self.records.lock().unwrap().clone()
    }

    /// Return the number of recorded decisions.
    pub fn len(&self) -> usize {
        self.records.lock().unwrap().len()
    }

    /// Whether any decisions have been recorded.
    pub fn is_empty(&self) -> bool {
        self.records.lock().unwrap().is_empty()
    }
}

impl AuditSink for InMemoryAudit {
    fn record(&self, record: &DecisionRecord) -> Result<()> {
        self.records.lock().unwrap().push(record.clone());
        Ok(())
    }
}
