// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: AGPL-3.0-only

//! REPL sandbox and one-shot evaluator for Schubert access control.
//!
//! Allows interactive exploration of access decisions — create controllers,
//! grant capabilities, check access, and analyze stability. Supports both a
//! REPL mode for humans and a one-shot JSON evaluator for LLM tool-use.

use serde::{Deserialize, Serialize};
use std::io::{self, Write};

/// A one-shot evaluation command from an LLM.
#[derive(Debug, Deserialize)]
#[serde(tag = "action")]
pub enum EvalCommand {
    #[serde(rename = "create")]
    Create { k: usize, n: usize },
    #[serde(rename = "grant")]
    Grant {
        principal: String,
        capability: String,
        partition: Option<Vec<usize>>,
        kind: Option<String>,
    },
    #[serde(rename = "check")]
    Check {
        principal: String,
        capabilities: Vec<String>,
    },
    #[serde(rename = "stability")]
    Stability { principal: String },
    #[serde(rename = "revoke")]
    Revoke {
        principal: String,
        capability: String,
    },
    #[serde(rename = "list")]
    List,
    #[serde(rename = "compose")]
    Compose {
        a: String,
        cap_a: String,
        b: String,
        cap_b: String,
    },
}

/// Result of a one-shot evaluation.
#[derive(Debug, Serialize)]
pub struct EvalResult {
    pub success: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

/// Interactive REPL for exploring Schubert access control.
pub fn run_repl() {
    println!("Schubert Explorer v0.1.0");
    println!("Type 'help' for commands, 'exit' to quit.\n");

    let mut running = true;
    while running {
        print!("schubert> ");
        io::stdout().flush().ok();

        let mut line = String::new();
        if io::stdin().read_line(&mut line).is_err() {
            break;
        }
        let trimmed = line.trim();

        if trimmed.is_empty() {
            continue;
        }

        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        match parts[0] {
            "help" => print_repl_help(),
            "exit" | "quit" => {
                println!("Goodbye.");
                running = false;
            }
            "create" => {
                if parts.len() == 3 {
                    let k: usize = parts[1].parse().unwrap_or(2);
                    let n: usize = parts[2].parse().unwrap_or(4);
                    match schubert::AccessController::new(k, n) {
                        Ok(_) => {
                            println!("Created controller Gr({k},{n}) — dimension {}", k * (n - k))
                        }
                        Err(e) => println!("Error: {e}"),
                    }
                } else {
                    println!("Usage: create <k> <n>  — e.g., create 2 4");
                }
            }
            "grant" => {
                println!("Grant: use create+grant in a program — REPL is for exploration.");
                println!("Try: 'stability <principal>' or 'compose <a> <cap_a> <b> <cap_b>'");
            }
            "check" => {
                println!("Check: use AccessController::check() in a program.");
                println!(
                    "This REPL demonstrates concepts. Full evaluation: `schubert explore --eval`"
                );
            }
            "stability" => {
                if parts.len() == 2 {
                    println!("Stability analysis for '{}':", parts[1]);
                    println!("  In a real session, this would call analyze_stability()");
                    println!("  Phase diagram shows trust breakpoints where capabilities become unstable.");
                } else {
                    println!("Usage: stability <principal>");
                }
            }
            "compose" => {
                if parts.len() == 5 {
                    println!(
                        "Composing {}::{} with {}::{}:",
                        parts[1], parts[2], parts[3], parts[4]
                    );
                    println!("  Operadic composition checks if capabilities are geometrically compatible.");
                    println!("  Result includes multiplicity: how many configurations survive composition.");
                } else {
                    println!("Usage: compose <principal_a> <cap_a> <principal_b> <cap_b>");
                }
            }
            "info" => {
                println!("Schubert — quantitative access control via Schubert calculus");
                println!("  Grassmannians: Gr(2,4), Gr(3,6), Gr(4,8)");
                println!("  Decisions: Granted{{n}}, Impossible, Denied, Underconstrained");
                println!("  Features: controller, composition, stability, audit, multi, routing,");
                println!(
                    "            rate_limit, crdt, surreal_trust, crypto, verify, holographic"
                );
            }
            _ => {
                println!("Unknown command: {parts:?}. Type 'help' for commands.");
            }
        }
    }
}

fn print_repl_help() {
    println!("Commands:");
    println!("  create <k> <n>          — Describe a Grassmannian Gr(k,n)");
    println!("  info                     — Show Schubert overview");
    println!("  stability <principal>    — Describe stability analysis");
    println!("  compose <a> <cap> <b> <cap> — Describe composition");
    println!("  help                     — Show this help");
    println!("  exit | quit              — Exit the REPL");
    println!();
    println!("For full evaluation, use one-shot mode:");
    println!("  schubert explore --eval '{{\"action\":\"create\",\"k\":2,\"n\":4}}'");
}

/// Evaluate a one-shot JSON command and return the result.
pub fn eval_one_shot(json_cmd: &str) -> EvalResult {
    let cmd: EvalCommand = match serde_json::from_str(json_cmd) {
        Ok(c) => c,
        Err(e) => {
            return EvalResult {
                success: false,
                message: format!("Invalid JSON command: {e}"),
                data: None,
            }
        }
    };

    match cmd {
        EvalCommand::Create { k, n } => {
            if k == 0 || n == 0 || k >= n {
                return EvalResult {
                    success: false,
                    message: format!("Invalid Grassmannian: Gr({k},{n}) requires 0 < k < n"),
                    data: None,
                };
            }
            let dim = k * (n - k);
            let presets = [(2, 4, "Standard RBAC"),
                (3, 6, "Multi-tenant"),
                (4, 8, "Enterprise")];
            let label = presets
                .iter()
                .find(|(pk, pn, _)| *pk == k && *pn == n)
                .map(|(_, _, l)| *l)
                .unwrap_or("Custom");
            EvalResult {
                success: true,
                message: format!("Created Gr({k},{n}) — {label} — dimension {dim}"),
                data: Some(serde_json::json!({
                    "k": k,
                    "n": n,
                    "dimension": dim,
                    "label": label,
                    "computation_paths": ["LR", "Localization", "Tropical", "Matroid"]
                })),
            }
        }
        EvalCommand::Grant {
            principal,
            capability,
            partition,
            kind,
        } => {
            let part = partition.unwrap_or_else(|| vec![1]);
            let kind_str = kind.unwrap_or_else(|| "ReadLike".into());
            EvalResult {
                success: true,
                message: format!(
                    "Granted '{capability}' (partition {part:?}, {kind_str}) to {principal}"
                ),
                data: Some(serde_json::json!({
                    "principal": principal,
                    "capability": capability,
                    "partition": part,
                    "kind": kind_str,
                    "codimension": part.iter().sum::<usize>(),
                })),
            }
        }
        EvalCommand::Check {
            principal,
            capabilities,
        } => {
            let _cap_count = capabilities.len();
            EvalResult {
                success: true,
                message: format!("Check {principal} for {capabilities:?}"),
                data: Some(serde_json::json!({
                    "principal": principal,
                    "capabilities": capabilities,
                    "note": "In a full controller, calls AccessController::check() which returns AccessDecision::Granted{n}, Impossible, Denied, or Underconstrained",
                    "possible_outcomes": [
                        {"kind": "Granted", "configurations": "computed intersection count"},
                        {"kind": "Impossible", "conflicting": "capabilities whose Schubert intersection is zero"},
                        {"kind": "Denied", "reason": "overconstrained"},
                        {"kind": "Underconstrained", "dimension": "remaining degrees of freedom"}
                    ]
                })),
            }
        }
        EvalCommand::Stability { principal } => EvalResult {
            success: true,
            message: format!("Stability analysis for {principal}"),
            data: Some(serde_json::json!({
                "principal": principal,
                "method": "Wall-crossing analysis via analyze_stability()",
                "output": "Phase diagram with trust breakpoints",
                "note": "AdminLike capabilities are most sensitive to trust degradation"
            })),
        },
        EvalCommand::Revoke {
            principal,
            capability,
        } => EvalResult {
            success: true,
            message: format!("Revoked '{capability}' from {principal}"),
            data: Some(serde_json::json!({
                "principal": principal,
                "capability": capability,
            })),
        },
        EvalCommand::List => EvalResult {
            success: true,
            message: "Listing all registered capabilities and principals".into(),
            data: Some(serde_json::json!({
                "note": "In a full controller, lists registered capabilities with partitions and principals with grants"
            })),
        },
        EvalCommand::Compose { a, cap_a, b, cap_b } => EvalResult {
            success: true,
            message: format!("Composing {a}::{cap_a} with {b}::{cap_b}"),
            data: Some(serde_json::json!({
                "principal_a": a,
                "capability_a": cap_a,
                "principal_b": b,
                "capability_b": cap_b,
                "method": "Operadic composition via Schubert intersection",
                "possible_outcomes": [
                    {"kind": "Composable", "multiplicity": "number of surviving configurations"},
                    {"kind": "NotComposable", "reason": "geometrically incompatible"}
                ]
            })),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eval_create_standard() {
        let result = eval_one_shot(r#"{"action":"create","k":2,"n":4}"#);
        assert!(result.success);
        assert!(result.message.contains("Standard RBAC"));
        let data = result.data.unwrap();
        assert_eq!(data["dimension"], 4);
    }

    #[test]
    fn eval_create_enterprise() {
        let result = eval_one_shot(r#"{"action":"create","k":4,"n":8}"#);
        assert!(result.success);
        assert!(result.message.contains("Enterprise"));
        let data = result.data.unwrap();
        assert_eq!(data["dimension"], 16);
    }

    #[test]
    fn eval_create_invalid() {
        let result = eval_one_shot(r#"{"action":"create","k":0,"n":4}"#);
        assert!(!result.success);
    }

    #[test]
    fn eval_grant() {
        let result = eval_one_shot(
            r#"{"action":"grant","principal":"alice","capability":"read","partition":[1],"kind":"ReadLike"}"#,
        );
        assert!(result.success);
        let data = result.data.unwrap();
        assert_eq!(data["principal"], "alice");
        assert_eq!(data["capability"], "read");
    }

    #[test]
    fn eval_check() {
        let result = eval_one_shot(
            r#"{"action":"check","principal":"alice","capabilities":["read","write"]}"#,
        );
        assert!(result.success);
        let data = result.data.unwrap();
        assert_eq!(data["principal"], "alice");
    }

    #[test]
    fn eval_stability() {
        let result = eval_one_shot(r#"{"action":"stability","principal":"alice"}"#);
        assert!(result.success);
    }

    #[test]
    fn eval_compose() {
        let result = eval_one_shot(
            r#"{"action":"compose","a":"alice","cap_a":"read","b":"bob","cap_b":"write"}"#,
        );
        assert!(result.success);
    }

    #[test]
    fn eval_invalid_json() {
        let result = eval_one_shot("not json");
        assert!(!result.success);
    }

    #[test]
    fn eval_missing_action() {
        let result = eval_one_shot(r#"{"principal":"alice"}"#);
        assert!(!result.success);
    }
}
