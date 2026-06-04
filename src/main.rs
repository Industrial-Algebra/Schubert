// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: AGPL-3.0-only

//! Schubert Discovery CLI — lightweight LLM function discovery for
//! quantitative access control via Schubert calculus.
//!
//! Subcommands:
//! - `discover`: Compact JSON schema of the API surface (token-efficient MCP alternative)
//! - `recommend`: Constraint-based config recommender (interactive or TOML input)
//! - `explore`: REPL sandbox or one-shot JSON evaluator

use clap::{Parser, Subcommand};

mod cli;

#[derive(Parser)]
#[command(
    name = "schubert",
    version = env!("CARGO_PKG_VERSION"),
    about = "Quantitative access control discovery CLI",
    long_about = "A lightweight LLM function-discovery CLI for Schubert calculus-based access control. \
                  Emits compact JSON schemas, recommends optimal configurations, and provides \
                  an interactive sandbox for exploring access decisions."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Emit a compact JSON schema of Schubert's API surface for LLM discovery
    Discover {
        /// Output format: json (compact) or md (markdown)
        #[arg(long, default_value = "json")]
        format: String,
        /// Filter by feature name (e.g., "crypto", "surreal", "policy")
        #[arg(long)]
        feature: Option<String>,
        /// Filter by module name (e.g., "controller", "routing")
        #[arg(long)]
        module: Option<String>,
    },

    /// Recommend optimal Schubert configuration from constraints
    Recommend {
        /// Path to a TOML constraint file (batch mode)
        #[arg(short, long)]
        input: Option<String>,
    },

    /// Interactive sandbox or one-shot evaluator for access decisions
    Explore {
        /// One-shot JSON evaluation command
        #[arg(long)]
        eval: Option<String>,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Discover {
            format,
            feature,
            module,
        } => {
            let json = if feature.is_some() || module.is_some() {
                cli::discover::api_catalog_filtered(feature.as_deref(), module.as_deref())
            } else {
                cli::discover::api_catalog_json()
            };

            match format.as_str() {
                "md" => {
                    // Parse JSON and emit as simple markdown
                    if let Ok(entries) = serde_json::from_str::<Vec<serde_json::Value>>(&json) {
                        println!("# Schubert API Catalog\n");
                        for e in &entries {
                            let name = e["name"].as_str().unwrap_or("?");
                            let kind = e["kind"].as_str().unwrap_or("?");
                            let sig = e["signature"].as_str().unwrap_or("");
                            let desc = e["description"].as_str().unwrap_or("");
                            let feat = e["feature"].as_str().unwrap_or("");
                            let feat_str = if feat.is_empty() {
                                String::new()
                            } else {
                                format!(" `[{feat}]`")
                            };
                            println!("### `{name}`{feat_str}");
                            println!("**{kind}** — {desc}");
                            println!("```rust\n{sig}\n```\n");
                        }
                    } else {
                        eprintln!("Error: could not parse API catalog");
                    }
                }
                _ => {
                    println!("{json}");
                }
            }
        }

        Commands::Recommend { input } => {
            let recommendation = if let Some(_input_path) = input {
                #[cfg(not(feature = "policy"))]
                {
                    eprintln!("Error: --input requires the 'policy' feature. Rebuild with --features policy");
                    std::process::exit(1);
                }
                #[cfg(feature = "policy")]
                {
                    match std::fs::read_to_string(&input_path) {
                        Ok(toml_str) => {
                            match toml::from_str::<cli::recommend::AccessConstraints>(&toml_str) {
                                Ok(constraints) => cli::recommend::recommend(&constraints),
                                Err(e) => {
                                    eprintln!("Error parsing TOML constraints: {e}");
                                    std::process::exit(1);
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Error reading file '{input_path}': {e}");
                            std::process::exit(1);
                        }
                    }
                }
            } else {
                match cli::recommend::interactive_recommend() {
                    Ok(r) => r,
                    Err(e) => {
                        eprintln!("Error: {e}");
                        std::process::exit(1);
                    }
                }
            };

            println!(
                "{}",
                serde_json::to_string_pretty(&recommendation).unwrap_or_default()
            );
        }

        Commands::Explore { eval } => {
            if let Some(cmd) = eval {
                let result = cli::explore::eval_one_shot(&cmd);
                println!(
                    "{}",
                    serde_json::to_string_pretty(&result).unwrap_or_default()
                );
            } else {
                cli::explore::run_repl();
            }
        }
    }
}
