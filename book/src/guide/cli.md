# CLI Discovery Tool

Schubert includes a lightweight CLI for LLM agents to discover and use its API.
Three subcommands cover the full lifecycle.

## Install

```bash
cargo install schubert
```

## `schubert discover` — API Catalog

Compact JSON schema of the full API surface (~200-500 tokens).

```bash
# Full catalog
schubert discover

# Filter by feature
schubert discover --feature crypto

# Filter by module
schubert discover --module routing

# Markdown output
schubert discover --format md
```

## `schubert recommend` — Config Recommender

```bash
# Interactive mode
schubert recommend

# Batch mode (LLM automation)
schubert recommend --input constraints.toml
```

Recommends optimal Gr(k,n), computation path, and feature flags given constraints
like number of roles, domains, audit requirements, and trust model.

## `schubert explore` — Decision Sandbox

```bash
# REPL mode
schubert explore

# One-shot evaluator (LLM tool-calling)
schubert explore --eval '{"action":"create","k":2,"n":4}'
schubert explore --eval '{"action":"check","principal":"alice","capabilities":["read","write"]}'
```

Supports actions: `create`, `grant`, `check`, `stability`, `compose`, `revoke`, `list`.

For the full guide, see [CLI Guide](../cli.md).
