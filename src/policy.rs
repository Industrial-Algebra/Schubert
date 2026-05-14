// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: AGPL-3.0-only

//! Declarative policy language for Schubert access control.
//!
//! The policy module enables defining capabilities, principals, and grants
//! in TOML format — "policy as code" with geometric guarantees.
//!
//! # Format
//!
//! ```toml
//! [grassmannian]
//! k = 2
//! n = 4
//!
//! [capabilities.read_data]
//! partition = [1]
//! kind = "ReadLike"
//! label = "Read data"
//! description = "Allows reading data records"
//!
//! [capabilities.write_data]
//! partition = [2]
//! kind = "WriteLike"
//! label = "Write data"
//!
//! [principals.alice]
//! grants = ["read_data", "write_data"]
//!
//! [principals.bob]
//! grants = ["read_data"]
//! ```
//!
//! # Loading
//!
//! ```
//! # #[cfg(feature = "policy")] {
//! use schubert::AccessController;
//!
//! let toml_str = r#"
//! [grassmannian]
//! k = 2
//! n = 4
//!
//! [capabilities.read]
//! partition = [1]
//! kind = "ReadLike"
//! label = "Read"
//! "#;
//!
//! let mut acl = AccessController::from_policy_toml(toml_str)?;
//! # }
//! # Ok::<(), schubert::SchubertError>(())
//! ```

use std::collections::BTreeMap;

use crate::{
    Capability, CapabilityKind, Result, SchubertError,
};

/// A complete access control policy in declarative form.
///
/// Maps directly to the TOML policy format. Deserialized via `serde`.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct PolicyConfig {
    /// Grassmannian parameters Gr(k,n).
    pub grassmannian: GrassmannianConfig,
    /// Named capabilities with their Schubert conditions.
    #[serde(default)]
    pub capabilities: BTreeMap<String, CapabilityConfig>,
    /// Named principals with their granted capabilities.
    #[serde(default)]
    pub principals: BTreeMap<String, PrincipalConfig>,
}

/// Grassmannian configuration.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct GrassmannianConfig {
    /// Dimension of subspaces.
    pub k: usize,
    /// Dimension of ambient space.
    pub n: usize,
}

/// A capability definition in the policy language.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct CapabilityConfig {
    /// Schubert partition defining the condition.
    pub partition: Vec<usize>,
    /// Semantic kind.
    pub kind: CapabilityKindConfig,
    /// Short human-readable label.
    pub label: String,
    /// Longer description (optional).
    #[serde(default)]
    pub description: String,
}

/// Capability kind in the policy language.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "PascalCase")]
pub enum CapabilityKindConfig {
    /// Read-like capability (typically codimension 1).
    ReadLike,
    /// Write-like capability (typically codimension 2).
    WriteLike,
    /// Administrative capability (typically point class).
    AdminLike,
    /// Custom capability with arbitrary partition semantics.
    Custom,
}

impl From<CapabilityKindConfig> for CapabilityKind {
    fn from(kind: CapabilityKindConfig) -> Self {
        match kind {
            CapabilityKindConfig::ReadLike => CapabilityKind::ReadLike,
            CapabilityKindConfig::WriteLike => CapabilityKind::WriteLike,
            CapabilityKindConfig::AdminLike => CapabilityKind::AdminLike,
            CapabilityKindConfig::Custom => CapabilityKind::Custom,
        }
    }
}

impl From<CapabilityKind> for CapabilityKindConfig {
    fn from(kind: CapabilityKind) -> Self {
        match kind {
            CapabilityKind::ReadLike => CapabilityKindConfig::ReadLike,
            CapabilityKind::WriteLike => CapabilityKindConfig::WriteLike,
            CapabilityKind::AdminLike => CapabilityKindConfig::AdminLike,
            CapabilityKind::Custom => CapabilityKindConfig::Custom,
        }
    }
}

/// A principal definition in the policy language.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct PrincipalConfig {
    /// Capability IDs granted to this principal.
    #[serde(default)]
    pub grants: Vec<String>,
}

// ═══════════════════════════════════════════════════════════════════════════
// Parsing and Validation
// ═══════════════════════════════════════════════════════════════════════════

impl PolicyConfig {
    /// Parse a policy from a TOML string.
    pub fn from_toml(toml_str: &str) -> std::result::Result<Self, toml::de::Error> {
        toml::from_str(toml_str)
    }

    /// Serialize a policy to a TOML string.
    pub fn to_toml(&self) -> std::result::Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }

    /// Validate the policy against the Grassmannian.
    ///
    /// Checks:
    /// - Grassmannian parameters are valid (k ≥ 1, n ≥ 2, k < n)
    /// - All capability partitions fit in the Grassmannian
    /// - All principal grants reference registered capabilities
    pub fn validate(&self) -> Result<()> {
        let (k, n) = (self.grassmannian.k, self.grassmannian.n);

        // Validate Grassmannian
        if k == 0 || n < 2 || k >= n {
            return Err(SchubertError::InvalidGrassmannian {
                k,
                n,
                reason: "require k ≥ 1, n ≥ 2, k < n".into(),
            });
        }

        // Validate capability partitions
        for (name, cap) in &self.capabilities {
            let dim = k * (n - k);
            let codim: usize = cap.partition.iter().sum();

            if codim > dim {
                return Err(SchubertError::InvalidPartition {
                    partition: cap.partition.clone(),
                    k,
                    n,
                    reason: format!(
                        "capability '{name}': codimension {codim} exceeds Grassmannian dimension {dim}"
                    ),
                });
            }

            // Check partition is weakly decreasing
            for w in cap.partition.windows(2) {
                if w[0] < w[1] {
                    return Err(SchubertError::InvalidPartition {
                        partition: cap.partition.clone(),
                        k,
                        n,
                        reason: format!(
                            "capability '{name}': partition not weakly decreasing ({:?})",
                            cap.partition
                        ),
                    });
                }
            }

            // Check partition fits in k×(n-k) box
            if cap.partition.len() > k {
                return Err(SchubertError::InvalidPartition {
                    partition: cap.partition.clone(),
                    k,
                    n,
                    reason: format!(
                        "capability '{name}': partition has {} parts, exceeds k={k}",
                        cap.partition.len()
                    ),
                });
            }
            if let Some(&first) = cap.partition.first() {
                if first > n - k {
                    return Err(SchubertError::InvalidPartition {
                        partition: cap.partition.clone(),
                        k,
                        n,
                        reason: format!(
                            "capability '{name}': first part {first} exceeds n-k={}",
                            n - k
                        ),
                    });
                }
            }
        }

        // Validate principal grants reference existing capabilities
        for (name, principal) in &self.principals {
            for grant in &principal.grants {
                if !self.capabilities.contains_key(grant) {
                    return Err(SchubertError::CapabilityNotFound(format!(
                        "principal '{name}' grants '{grant}', but capability '{grant}' is not defined"
                    )));
                }
            }
        }

        Ok(())
    }

    /// Apply the policy to an access controller.
    ///
    /// Creates all capabilities and principals, and grants the specified
    /// capabilities to each principal.
    pub fn apply(&self, acl: &mut crate::AccessController) -> Result<()> {
        self.validate()?;

        // Register capabilities
        for (name, cap_config) in &self.capabilities {
            let cap = Capability::with_description(
                name.clone(),
                cap_config.label.clone(),
                cap_config.description.clone(),
                cap_config.partition.clone(),
                CapabilityKind::from(cap_config.kind.clone()),
            );
            acl.register_capability(cap)?;
        }

        // Create principals and grant capabilities
        for (name, principal_config) in &self.principals {
            let pid = acl.create_principal(name.clone())?;
            for grant in &principal_config.grants {
                acl.grant(&pid, grant)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PrincipalId;

    const BASIC_POLICY: &str = r#"
[grassmannian]
k = 2
n = 4

[capabilities.read]
partition = [1]
kind = "ReadLike"
label = "Read data"

[capabilities.write]
partition = [2]
kind = "WriteLike"
label = "Write data"

[principals.alice]
grants = ["read", "write"]

[principals.bob]
grants = ["read"]
"#;

    #[test]
    fn parse_basic_policy() {
        let config = PolicyConfig::from_toml(BASIC_POLICY).unwrap();
        assert_eq!(config.grassmannian.k, 2);
        assert_eq!(config.grassmannian.n, 4);
        assert_eq!(config.capabilities.len(), 2);
        assert_eq!(config.principals.len(), 2);
    }

    #[test]
    fn validate_basic_policy() {
        let config = PolicyConfig::from_toml(BASIC_POLICY).unwrap();
        config.validate().unwrap();
    }

    #[test]
    fn apply_basic_policy() {
        let config = PolicyConfig::from_toml(BASIC_POLICY).unwrap();
        let mut acl = crate::AccessController::new(2, 4).unwrap();
        config.apply(&mut acl).unwrap();

        let alice = PrincipalId::new("alice");
        let bob = PrincipalId::new("bob");

        let alice_principal = acl.principal(&alice).unwrap();
        let bob_principal = acl.principal(&bob).unwrap();

        assert!(alice_principal.holds("read"));
        assert!(alice_principal.holds("write"));
        assert!(bob_principal.holds("read"));
        assert!(!bob_principal.holds("write"));
    }

    #[test]
    fn validate_rejects_invalid_grassmannian() {
        let bad = r#"
[grassmannian]
k = 3
n = 3
"#;
        let config = PolicyConfig::from_toml(bad).unwrap();
        assert!(config.validate().is_err());
    }

    #[test]
    fn validate_rejects_partition_too_large() {
        let bad = r#"
[grassmannian]
k = 2
n = 4

[capabilities.bad]
partition = [5]
kind = "ReadLike"
label = "Too large"
"#;
        let config = PolicyConfig::from_toml(bad).unwrap();
        let err = config.validate().unwrap_err();
        assert!(format!("{err}").contains("codimension"));
    }

    #[test]
    fn validate_rejects_non_weakly_decreasing() {
        let bad = r#"
[grassmannian]
k = 2
n = 4

[capabilities.bad]
partition = [1, 2]
kind = "ReadLike"
label = "Not decreasing"
"#;
        let config = PolicyConfig::from_toml(bad).unwrap();
        let err = config.validate().unwrap_err();
        assert!(format!("{err}").contains("not weakly decreasing"));
    }

    #[test]
    fn validate_rejects_missing_capability_reference() {
        let bad = r#"
[grassmannian]
k = 2
n = 4

[principals.alice]
grants = ["nonexistent"]
"#;
        let config = PolicyConfig::from_toml(bad).unwrap();
        let err = config.validate().unwrap_err();
        assert!(format!("{err}").contains("not defined"));
    }

    #[test]
    fn roundtrip_policy_toml() {
        let config = PolicyConfig::from_toml(BASIC_POLICY).unwrap();
        let toml_out = config.to_toml().unwrap();
        let config2 = PolicyConfig::from_toml(&toml_out).unwrap();
        assert_eq!(config2.capabilities.len(), 2);
        assert_eq!(config2.principals.len(), 2);
    }

    #[test]
    fn empty_capabilities_and_principals() {
        let minimal = r#"
[grassmannian]
k = 2
n = 4
"#;
        let config = PolicyConfig::from_toml(minimal).unwrap();
        config.validate().unwrap();
        assert!(config.capabilities.is_empty());
        assert!(config.principals.is_empty());
    }
}
