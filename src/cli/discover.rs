// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: AGPL-3.0-only

//! Compact JSON schema describing Schubert's API surface for LLM discovery.
//!
//! Emits a token-efficient catalog of types, functions, traits, and constants
//! that an LLM agent can use to understand what Schubert provides and how to
//! call it. Designed as a lightweight alternative to MCP function discovery.

use serde::Serialize;

/// A compact API entry describing one callable item.
#[derive(Serialize, Clone)]
struct ApiEntry {
    /// Function or type name
    name: &'static str,
    /// "function", "struct", "enum", "trait", "const"
    kind: &'static str,
    /// Rust signature (compact)
    signature: &'static str,
    /// One-line description
    description: &'static str,
    /// Feature gate required (empty if always available)
    feature: &'static str,
    /// Module path
    module: &'static str,
}

/// Generate the full API catalog as a JSON string.
pub fn api_catalog_json() -> String {
    let catalog = build_catalog();
    serde_json::to_string_pretty(&catalog).unwrap_or_else(|_| "[]".into())
}

/// Generate a filtered catalog by feature or module name.
pub fn api_catalog_filtered(feature: Option<&str>, module: Option<&str>) -> String {
    let catalog = build_catalog();
    let filtered: Vec<ApiEntry> = catalog
        .iter()
        .filter(|e| {
            let feat_ok = feature.map_or(true, |f| e.feature.contains(f));
            let mod_ok = module.map_or(true, |m| e.module.contains(m));
            feat_ok && mod_ok
        })
        .cloned()
        .collect();
    serde_json::to_string_pretty(&filtered).unwrap_or_else(|_| "[]".into())
}

fn build_catalog() -> Vec<ApiEntry> {
    vec![
        // --- controller.rs ---
        ApiEntry {
            name: "AccessController",
            kind: "struct",
            signature: "AccessController::new(k: usize, n: usize) -> Result<Self>",
            description: "Create a new access controller on Gr(k,n)",
            feature: "",
            module: "controller",
        },
        ApiEntry {
            name: "AccessController::register_capability",
            kind: "function",
            signature: "fn register_capability(&mut self, cap: Capability) -> Result<()>",
            description: "Register a new capability (Schubert condition)",
            feature: "",
            module: "controller",
        },
        ApiEntry {
            name: "AccessController::create_principal",
            kind: "function",
            signature: "fn create_principal(&mut self, id: &str) -> Result<PrincipalId>",
            description: "Create a new principal identity",
            feature: "",
            module: "controller",
        },
        ApiEntry {
            name: "AccessController::grant",
            kind: "function",
            signature: "fn grant(&mut self, principal: &PrincipalId, capability: &str) -> Result<()>",
            description: "Grant a capability to a principal",
            feature: "",
            module: "controller",
        },
        ApiEntry {
            name: "AccessController::revoke",
            kind: "function",
            signature: "fn revoke(&mut self, principal: &PrincipalId, capability: &str) -> Result<()>",
            description: "Revoke a capability from a principal",
            feature: "",
            module: "controller",
        },
        ApiEntry {
            name: "AccessController::check",
            kind: "function",
            signature: "fn check(&self, principal: &PrincipalId, required: &[&str]) -> Result<AccessDecision>",
            description: "Check if a principal satisfies a set of capabilities — returns quantitative decision",
            feature: "",
            module: "controller",
        },
        ApiEntry {
            name: "AccessController::check_batch",
            kind: "function",
            signature: "fn check_batch(&self, queries: &[(PrincipalId, Vec<&str>)]) -> Result<Vec<AccessDecision>>",
            description: "Batch-check multiple principals against capability sets (parallel feature)",
            feature: "parallel",
            module: "controller",
        },
        // --- capability.rs ---
        ApiEntry {
            name: "Capability",
            kind: "struct",
            signature: "Capability::new(id: &str, label: &str, partition: Vec<usize>, kind: CapabilityKind) -> Self",
            description: "A Schubert condition with a partition and kind",
            feature: "",
            module: "capability",
        },
        ApiEntry {
            name: "CapabilityKind",
            kind: "enum",
            signature: "ReadLike | WriteLike | AdminLike | Custom",
            description: "Semantic category of a capability affecting trust sensitivity",
            feature: "",
            module: "capability",
        },
        // --- decision.rs ---
        ApiEntry {
            name: "AccessDecision",
            kind: "enum",
            signature: "Granted { configurations: usize } | Impossible { conflicting: Vec<String> } | Denied | Underconstrained { dimension: usize }",
            description: "Quantitative access decision — not just allow/deny",
            feature: "",
            module: "decision",
        },
        ApiEntry {
            name: "ComputationPath",
            kind: "enum",
            signature: "LR | Localization | Tropical | Matroid",
            description: "Which Schubert calculus engine to use",
            feature: "",
            module: "decision",
        },
        ApiEntry {
            name: "AccessContext",
            kind: "struct",
            signature: "AccessContext { resource: Option<String>, time_budget_ms: Option<u64>, required_trust: f64 }",
            description: "Context-aware access check with resource scoping and trust requirements",
            feature: "",
            module: "decision",
        },
        // --- composition.rs ---
        ApiEntry {
            name: "compose",
            kind: "function",
            signature: "fn compose(acl: &AccessController, a: &PrincipalId, cap_a: &str, b: &PrincipalId, cap_b: &str) -> Result<CompositionResult>",
            description: "Operadic composition — check if two principals' capabilities are composable",
            feature: "",
            module: "composition",
        },
        // --- stability.rs ---
        ApiEntry {
            name: "analyze_stability",
            kind: "function",
            signature: "fn analyze_stability(acl: &AccessController, principal: &PrincipalId) -> Result<StabilityReport>",
            description: "Wall-crossing stability analysis — find trust breakpoints",
            feature: "",
            module: "stability",
        },
        // --- audit.rs ---
        ApiEntry {
            name: "AuditSink",
            kind: "trait",
            signature: "fn record(&self, record: &DecisionRecord) -> Result<()>",
            description: "Pluggable audit sink for recording access decisions",
            feature: "std",
            module: "audit",
        },
        // --- multi.rs ---
        ApiEntry {
            name: "MultiController",
            kind: "struct",
            signature: "MultiController::new() -> Self",
            description: "Manage multiple Grassmannian domains with cross-domain capability translation",
            feature: "",
            module: "multi",
        },
        ApiEntry {
            name: "MultiController::check_cross_domain",
            kind: "function",
            signature: "fn check_cross_domain(&self, principal: &PrincipalId, caps: &[&str], from: &DomainId, to: &DomainId) -> Result<AccessDecision>",
            description: "Check access across different Grassmannian domains",
            feature: "",
            module: "multi",
        },
        // --- rate_limit.rs ---
        ApiEntry {
            name: "RateLimiter",
            kind: "struct",
            signature: "RateLimiter::new(refill_rate: f64, max_tokens: f64) -> Self",
            description: "Token-bucket rate limiter scaled by Schubert intersection numbers",
            feature: "",
            module: "rate_limit",
        },
        // --- routing.rs ---
        ApiEntry {
            name: "RouteTable",
            kind: "struct",
            signature: "RouteTable::new(k: usize, n: usize) -> Self",
            description: "Geometric routing table — find paths through Schubert conditions",
            feature: "",
            module: "routing",
        },
        // --- crdt.rs ---
        ApiEntry {
            name: "CrdtState",
            kind: "struct",
            signature: "CrdtState::new() -> Self",
            description: "Eventually-consistent CRDT state for distributed grants",
            feature: "",
            module: "crdt",
        },
        // --- policy.rs ---
        ApiEntry {
            name: "AccessController::from_policy_toml",
            kind: "function",
            signature: "fn from_policy_toml(toml_str: &str) -> Result<Self>",
            description: "Load access controller configuration from a TOML policy file",
            feature: "policy",
            module: "policy",
        },
        ApiEntry {
            name: "AccessController::to_policy_toml",
            kind: "function",
            signature: "fn to_policy_toml(&self) -> Result<String>",
            description: "Serialize access controller to a TOML policy string",
            feature: "policy",
            module: "policy",
        },
        // --- crypto.rs ---
        ApiEntry {
            name: "CapabilityIssuer",
            kind: "struct",
            signature: "CapabilityIssuer::generate() -> Self",
            description: "Generate Ed25519 key pair for issuing signed capability tokens",
            feature: "crypto",
            module: "crypto",
        },
        ApiEntry {
            name: "CapabilityVerifier",
            kind: "struct",
            signature: "CapabilityVerifier::new(public_key: PublicKey) -> Self",
            description: "Verify Ed25519-signed capability tokens",
            feature: "crypto",
            module: "crypto",
        },
        // --- surreal_trust.rs ---
        ApiEntry {
            name: "SurrealTrust",
            kind: "struct",
            signature: "SurrealTrust::new(value: RationalSurreal) -> Self",
            description: "Exact surreal trust level using Amari RationalSurreal arithmetic",
            feature: "surreal",
            module: "surreal_trust",
        },
        // --- verify.rs ---
        ApiEntry {
            name: "Certified",
            kind: "struct",
            signature: "Certified<T> — a value with a Karpal proof obligation",
            description: "Trust boundary: a value certified by formal verification",
            feature: "karpal-verify",
            module: "verify",
        },
        // --- holographic.rs ---
        ApiEntry {
            name: "HolographicAccessControl",
            kind: "struct",
            signature: "HolographicAccessControl::new() -> Self",
            description: "Holographic memory access control via Minuet cosine similarity",
            feature: "holographic",
            module: "holographic",
        },
        // --- wasm.rs ---
        ApiEntry {
            name: "WasmController",
            kind: "struct",
            signature: "WasmController::new(k: usize, n: usize) -> Result<Self>",
            description: "WebAssembly access controller with JS bindings",
            feature: "wasm",
            module: "wasm",
        },
        // --- Constants ---
        ApiEntry {
            name: "GRASSMANNIAN_PRESETS",
            kind: "const",
            signature: "[(2,4), (3,6), (4,8)]",
            description: "Common Grassmannian presets: (2,4) for standard RBAC, (3,6) for multi-tenant, (4,8) for enterprise",
            feature: "",
            module: "controller",
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_is_valid_json() {
        let json = api_catalog_json();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        let arr = parsed.as_array().unwrap();
        assert!(!arr.is_empty(), "Catalog should not be empty");
    }

    #[test]
    fn catalog_entries_have_required_fields() {
        let json = api_catalog_json();
        let entries: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();
        for entry in &entries {
            assert!(entry["name"].is_string(), "Entry missing name");
            assert!(entry["kind"].is_string(), "Entry missing kind");
            assert!(entry["signature"].is_string(), "Entry missing signature");
            assert!(
                entry["description"].is_string(),
                "Entry missing description"
            );
        }
    }

    #[test]
    fn filter_by_feature() {
        let json = api_catalog_filtered(Some("crypto"), None);
        let entries: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();
        for entry in &entries {
            let feat = entry["feature"].as_str().unwrap();
            assert!(
                feat.contains("crypto"),
                "Filtered entry should contain 'crypto': {feat}"
            );
        }
    }

    #[test]
    fn discover_tool_callable() {
        // Integration test: validate the full discover catalog parses
        let json = api_catalog_json();
        let entries: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();
        // Should have at least 20 API entries
        assert!(
            entries.len() >= 20,
            "Catalog should have >= 20 entries, got {}",
            entries.len()
        );
        // All entries must be valid
        for entry in &entries {
            assert!(entry["name"].is_string());
            assert!(entry["kind"].is_string());
        }
    }
}
