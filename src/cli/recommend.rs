// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! Constraint-based config recommender for Schubert access control.
//!
//! Given a set of requirements (number of roles, domains, audit, crypto, trust model),
//! recommends the optimal Grassmannian, computation path, and feature flags.
//! Supports both interactive mode and TOML constraint file input.

use serde::{Deserialize, Serialize};

/// Input constraints for the recommender.
#[derive(Debug, Deserialize, Serialize)]
pub struct AccessConstraints {
    /// Number of distinct user roles needed
    #[serde(default = "default_num_roles")]
    pub num_roles: usize,
    /// Number of distinct security domains / tenants
    #[serde(default)]
    pub num_domains: usize,
    /// Whether audit logging is required
    #[serde(default)]
    pub audit_required: bool,
    /// Whether cryptographic capability tokens are needed
    #[serde(default)]
    pub crypto_required: bool,
    /// Trust model: "discrete", "continuous", or "surreal"
    #[serde(default = "default_trust_model")]
    pub trust_model: String,
    /// Whether parallel batch operations are needed
    #[serde(default)]
    pub parallel_required: bool,
    /// Whether formal verification (Karpal) is needed
    #[serde(default)]
    pub verification_required: bool,
    /// Whether holographic memory integration is needed
    #[serde(default)]
    pub holographic_required: bool,
    /// Whether policy-as-code (TOML) is desired
    #[serde(default)]
    pub policy_required: bool,
    /// Whether WebAssembly deployment is needed
    #[serde(default)]
    pub wasm_required: bool,
    /// Maximum expected concurrent principals
    #[serde(default = "default_max_principals")]
    pub max_principals: usize,
    /// Whether cross-domain capability translation is needed
    #[serde(default)]
    pub cross_domain: bool,
}

fn default_num_roles() -> usize {
    3
}
fn default_trust_model() -> String {
    "discrete".into()
}
fn default_max_principals() -> usize {
    100
}

/// Output recommendation from the recommender.
#[derive(Debug, Serialize)]
pub struct Recommendation {
    /// Recommended Grassmannian dimensions
    pub grassmannian: (usize, usize),
    /// Policy space dimension k(n-k)
    pub policy_dimension: usize,
    /// Recommended computation path
    pub computation_path: &'static str,
    /// Required Cargo feature flags
    pub features: Vec<&'static str>,
    /// Description of why this was chosen
    pub rationale: String,
    /// Example Cargo.toml dependency line
    pub cargo_toml: String,
    /// Example Rust initialization code
    pub example_code: String,
}

/// Generate a recommendation from constraints.
pub fn recommend(constraints: &AccessConstraints) -> Recommendation {
    let (k, n) = select_grassmannian(constraints);
    let path = select_computation_path(constraints);
    let features = select_features(constraints);
    let policy_dimension = k * (n - k);

    let rationale = build_rationale(constraints, (k, n), path, &features);

    let cargo_toml = if features.is_empty() {
        "schubert = \"0.1\"".to_string()
    } else {
        format!(
            "schubert = {{ version = \"0.1\", features = [{}] }}",
            features
                .iter()
                .map(|f| format!("\"{f}\""))
                .collect::<Vec<_>>()
                .join(", ")
        )
    };

    let example_code = build_example_code(k, n, &features);

    Recommendation {
        grassmannian: (k, n),
        policy_dimension,
        computation_path: path,
        features,
        rationale,
        cargo_toml,
        example_code,
    }
}

fn select_grassmannian(c: &AccessConstraints) -> (usize, usize) {
    let total_dim = c.num_roles + c.num_domains;
    if c.cross_domain || c.num_domains > 2 || total_dim > 8 {
        // Enterprise-scale
        (4, 8) // dim 16
    } else if total_dim > 5 || c.num_domains > 1 {
        // Multi-tenant
        (3, 6) // dim 9
    } else {
        // Standard RBAC
        (2, 4) // dim 4
    }
}

fn select_computation_path(c: &AccessConstraints) -> &'static str {
    if c.verification_required {
        "LR (best for verified environments)"
    } else if c.max_principals > 1000 {
        "Tropical (best for large-scale batch operations)"
    } else if c.parallel_required {
        "Matroid (best for parallel evaluation)"
    } else {
        "LR (default — balanced performance)"
    }
}

fn select_features(c: &AccessConstraints) -> Vec<&'static str> {
    let mut feats = vec!["std"];
    if c.audit_required {
        // std already included for audit
    }
    if c.crypto_required {
        feats.push("crypto");
    }
    if c.trust_model == "surreal" {
        feats.push("surreal");
    }
    if c.parallel_required {
        feats.push("parallel");
    }
    if c.policy_required {
        feats.push("policy");
    }
    if c.verification_required {
        feats.push("karpal-verify");
    }
    if c.holographic_required {
        feats.push("holographic");
    }
    if c.wasm_required {
        feats.push("wasm");
    }
    feats
}

fn build_rationale(
    c: &AccessConstraints,
    (k, n): (usize, usize),
    path: &str,
    features: &[&str],
) -> String {
    let mut r = format!(
        "Selected Gr({k},{n}) with dimension {} for {roles} roles",
        k * (n - k),
        roles = c.num_roles
    );
    if c.num_domains > 0 {
        r.push_str(&format!(" across {} domains", c.num_domains));
    }
    r.push_str(&format!(". Computation path: {path}."));
    r.push_str(&format!(" Features: [{}].", features.join(", ")));
    if c.crypto_required {
        r.push_str(" Cryptographic tokens enabled for proof-carrying capabilities.");
    }
    if c.trust_model == "surreal" {
        r.push_str(" Surreal trust arithmetic for infinitesimal trust resolution.");
    }
    r
}

fn build_example_code(k: usize, n: usize, features: &[&str]) -> String {
    let mut code = format!(
        "use schubert::AccessController;\n\
         \n\
         fn main() -> Result<(), Box<dyn std::error::Error>> {{\n\
         \x20   let mut acl = AccessController::new({k}, {n})?;\n"
    );

    if features.contains(&"policy") {
        code.push_str("    // Load from TOML policy file\n");
        code.push_str(
            "    let acl = AccessController::from_policy_toml(\n        &std::fs::read_to_string(\"policy.toml\")?\n    )?;\n",
        );
    }

    code.push_str("    let alice = acl.create_principal(\"alice\")?;\n");

    if features.contains(&"crypto") {
        code.push_str("    // Issue a cryptographic capability token\n");
        code.push_str("    let issuer = CapabilityIssuer::generate();\n");
        code.push_str("    let token = issuer.issue(\"alice\", \"read\")?;\n");
    }

    code.push_str("    acl.grant(&alice, \"read\")?;\n");
    code.push_str("    let decision = acl.check(&alice, &[\"read\"])?;\n");
    code.push_str("    println!(\"{decision:?}\");\n");
    code.push_str("    Ok(())\n}\n");
    code
}

/// Run an interactive recommendation session via stdin.
pub fn interactive_recommend() -> Result<Recommendation, String> {
    use std::io::{self, Write};

    let mut constraints = AccessConstraints {
        num_roles: 3,
        num_domains: 0,
        audit_required: false,
        crypto_required: false,
        trust_model: "discrete".into(),
        parallel_required: false,
        verification_required: false,
        holographic_required: false,
        policy_required: false,
        wasm_required: false,
        max_principals: 100,
        cross_domain: false,
    };

    let ask = |prompt: &str, default: &str| -> String {
        print!("{prompt} [{default}]: ");
        io::stdout().flush().ok();
        let mut line = String::new();
        io::stdin().read_line(&mut line).ok();
        let trimmed = line.trim().to_string();
        if trimmed.is_empty() {
            default.to_string()
        } else {
            trimmed
        }
    };

    let ask_bool = |prompt: &str, default: bool| -> bool {
        let def_str = if default { "y" } else { "n" };
        let ans = ask(prompt, def_str);
        ans.to_lowercase().starts_with('y')
    };

    println!("\n=== Schubert Configuration Recommender ===\n");

    constraints.num_roles = ask("Number of user roles", "3").parse().unwrap_or(3);
    constraints.num_domains = ask("Number of security domains (0 for single-domain)", "0")
        .parse()
        .unwrap_or(0);
    constraints.cross_domain = ask_bool(
        "Cross-domain capability translation?",
        constraints.num_domains > 1,
    );
    constraints.audit_required = ask_bool("Audit logging?", true);
    constraints.crypto_required = ask_bool("Cryptographic capability tokens?", false);
    constraints.policy_required = ask_bool("Policy-as-code (TOML files)?", true);

    let trust = ask("Trust model (discrete, continuous, surreal)", "discrete");
    constraints.trust_model = if trust.contains("surreal") {
        "surreal".into()
    } else if trust.contains("cont") {
        "continuous".into()
    } else {
        "discrete".into()
    };

    constraints.parallel_required = ask_bool(
        "Parallel batch operations?",
        constraints.max_principals > 500,
    );
    constraints.verification_required = ask_bool("Formal verification (Karpal)?", false);
    constraints.holographic_required = ask_bool("Holographic memory (Minuet)?", false);
    constraints.wasm_required = ask_bool("WebAssembly deployment?", false);
    constraints.max_principals = ask("Max concurrent principals", "100")
        .parse()
        .unwrap_or(100);

    println!();
    Ok(recommend(&constraints))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recommend_standard_rbac() {
        let c = AccessConstraints {
            num_roles: 3,
            ..AccessConstraints {
                num_roles: 0,
                num_domains: 0,
                audit_required: false,
                crypto_required: false,
                trust_model: "discrete".into(),
                parallel_required: false,
                verification_required: false,
                holographic_required: false,
                policy_required: false,
                wasm_required: false,
                max_principals: 100,
                cross_domain: false,
            }
        };
        let r = recommend(&c);
        assert_eq!(r.grassmannian, (2, 4));
        assert!(r.features.contains(&"std"));
    }

    #[test]
    fn recommend_enterprise() {
        let c = AccessConstraints {
            num_roles: 6,
            num_domains: 3,
            cross_domain: true,
            ..AccessConstraints {
                num_roles: 0,
                num_domains: 0,
                audit_required: false,
                crypto_required: false,
                trust_model: "discrete".into(),
                parallel_required: false,
                verification_required: false,
                holographic_required: false,
                policy_required: false,
                wasm_required: false,
                max_principals: 100,
                cross_domain: false,
            }
        };
        let r = recommend(&c);
        assert_eq!(r.grassmannian, (4, 8));
        assert_eq!(r.policy_dimension, 16);
    }

    #[test]
    fn recommend_with_crypto() {
        let c = AccessConstraints {
            num_roles: 3,
            crypto_required: true,
            ..AccessConstraints {
                num_roles: 0,
                num_domains: 0,
                audit_required: false,
                crypto_required: false,
                trust_model: "discrete".into(),
                parallel_required: false,
                verification_required: false,
                holographic_required: false,
                policy_required: false,
                wasm_required: false,
                max_principals: 100,
                cross_domain: false,
            }
        };
        let r = recommend(&c);
        assert!(r.features.contains(&"crypto"));
    }

    #[test]
    fn recommend_with_surreal() {
        let c = AccessConstraints {
            num_roles: 4,
            trust_model: "surreal".into(),
            ..AccessConstraints {
                num_roles: 0,
                num_domains: 0,
                audit_required: false,
                crypto_required: false,
                trust_model: "discrete".into(),
                parallel_required: false,
                verification_required: false,
                holographic_required: false,
                policy_required: false,
                wasm_required: false,
                max_principals: 100,
                cross_domain: false,
            }
        };
        let r = recommend(&c);
        assert!(r.features.contains(&"surreal"));
    }

    #[test]
    fn recommend_all_features() {
        let c = AccessConstraints {
            num_roles: 8,
            num_domains: 4,
            audit_required: true,
            crypto_required: true,
            trust_model: "surreal".into(),
            parallel_required: true,
            verification_required: true,
            holographic_required: true,
            policy_required: true,
            wasm_required: true,
            max_principals: 5000,
            cross_domain: true,
        };
        let r = recommend(&c);
        assert!(r.features.contains(&"crypto"));
        assert!(r.features.contains(&"surreal"));
        assert!(r.features.contains(&"parallel"));
        assert!(r.features.contains(&"policy"));
        assert!(r.features.contains(&"karpal-verify"));
        assert!(r.features.contains(&"holographic"));
        assert!(r.features.contains(&"wasm"));
    }
}
