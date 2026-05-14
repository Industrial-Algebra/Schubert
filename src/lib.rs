//! # Schubert — Quantitative Access Control via Schubert Calculus
//!
//! > "How many ways can this principal access this resource?"
//!
//! Schubert replaces boolean allow/deny with **geometric access control**.
//! Capabilities are Schubert conditions on a Grassmannian — access is
//! granted when the intersection is non-empty, and the intersection number
//! tells you exactly how many valid configurations exist.
//!
//! ## Key Features
//!
//! - **Quantitative decisions**: Not just "allowed" but "allowed in exactly N ways"
//! - **Impossibility detection**: Catches policies that are dimensionally valid
//!   but geometrically impossible (the σ₂·σ₁₁ = 0 case)
//! - **Operadic composition**: Compose principals via shared capabilities
//!   with computed multiplicities
//! - **Stability analysis**: Understand which capabilities become unstable
//!   as trust degrades (wall-crossing phase diagrams)
//! - **Zero runtime dependencies**: Only `amari-enumerative` for the math.
//!   Synchronous API. Embeddable anywhere.
//!
//! ## Quick Start
//!
//! ```
//! use schubert::{AccessController, Capability, CapabilityKind, AccessDecision};
//!
//! // Create an access controller for Gr(2,4) — 4-dimensional policy space
//! let mut acl = AccessController::new(2, 4)?;
//!
//! // Register capabilities (Schubert conditions)
//! acl.register_capability(Capability::new(
//!     "read:data", "Read data", vec![1], CapabilityKind::ReadLike,
//! ))?;
//! acl.register_capability(Capability::new(
//!     "write:data", "Write data", vec![2], CapabilityKind::WriteLike,
//! ))?;
//!
//! // Create a principal and grant capabilities
//! let alice = acl.create_principal("alice")?;
//! acl.grant(&alice, "read:data")?;
//! acl.grant(&alice, "write:data")?;
//!
//! // Check access — get a quantitative result
//! match acl.check(&alice, &["read:data", "write:data"])? {
//!     AccessDecision::Granted { configurations, path } => {
//!         println!("Access granted with {} configurations via {:?}", configurations, path);
//!     }
//!     AccessDecision::Impossible { conflicting } => {
//!         println!("Policy conflict: {:?} are geometrically incompatible", conflicting);
//!     }
//!     AccessDecision::Denied => println!("Access denied"),
//!     AccessDecision::Underconstrained { dimension } => {
//!         println!("Policy too permissive (dimension {})", dimension);
//!     }
//! }
//! # Ok::<(), schubert::SchubertError>(())
//! ```
//!
//! ## Grassmannian Selection
//!
//! | Gr(k,n) | Dimension | Use Case |
//! |---------|-----------|----------|
//! | Gr(1,2) | 1 | Simple binary access |
//! | Gr(1,3) | 2 | Read/write on one resource |
//! | Gr(2,4) | 4 | Standard RBAC (4 distinct conditions) |
//! | Gr(3,6) | 9 | Complex multi-tenant |
//! | Gr(4,8) | 16 | Enterprise policy space |
//!
//! ## What Schubert Is Not
//!
//! - An authentication system (identity is external)
//! - A network service (library only)
//! - A replacement for OAuth/OIDC (compatible alongside them)
//! - A key-value store or database
//!
//! ## Feature Flags
//!
//! - `std` (default) — Enables `std::collections::HashMap`, `SystemTime` timestamps,
//!   and thread-safe audit via `Mutex`. Disable for `no_std` environments.
//! - `serde` — Enables `Serialize`/`Deserialize` on key types for policy persistence.
//!
//! ## `no_std` Support
//!
//! Schubert targets `no_std` compatibility via the `alloc` crate. When the `std`
//! feature is disabled:
//!
//! - `HashMap` is replaced with `BTreeMap` from `alloc`
//! - `InMemoryAudit` is not thread-safe (single-threaded environments only)
//! - `now_millis()` returns 0 (no system clock available)
//! - Full `no_std` requires an allocator and `amari-enumerative` without `std`

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod audit;
pub mod capability;
pub mod composition;
/// Access controller managing principals, capabilities, and checks.
pub mod controller;
/// Access decision types — the quantitative result of every check.
pub mod decision;
pub mod error;
/// Compile-time phantom type markers from amari-enumerative.
pub mod phantom;
pub mod principal;
pub mod stability;

// Core types — everything you typically need
pub use audit::{AuditSink, DecisionRecord, InMemoryAudit};
pub use capability::{Capability, CapabilityId, CapabilityKind};
pub use composition::{compose, are_composable, CompositionResult};
pub use controller::AccessController;
pub use decision::{AccessDecision, ComputationPath};
pub use error::{Result, SchubertError};
pub use principal::{Principal, PrincipalId};
pub use stability::{
    analyze_stability, stable_capabilities_at, StabilityBreakpoint, StabilityReport, TrustLevel,
};
