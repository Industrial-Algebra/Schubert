# Schubert Discovery CLI

A lightweight, token-efficient CLI for LLM agents to discover and use Schubert's
quantitative access control capabilities. Three subcommands — `discover`, `recommend`,
and `explore` — cover the full lifecycle: learning the API, configuring a deployment,
and evaluating access decisions.

## Quick Start

```bash
cargo install schubert
schubert --help
```

## Subcommand Reference

### `discover` — API Surface Catalog

Emits a compact JSON schema of Schubert's public API. Designed as a lightweight
alternative to MCP function discovery — an LLM can read the full catalog in
~200-500 tokens and immediately understand what Schubert provides.

```bash
# Full catalog (JSON)
schubert discover

# Filter by feature gate
schubert discover --feature crypto

# Filter by module
schubert discover --module routing

# Markdown output (human-readable)
schubert discover --format md

# Combine filters
schubert discover --feature surreal --module trust
```

**Output format** (JSON):
```json
[
  {
    "name": "AccessController",
    "kind": "struct",
    "signature": "AccessController::new(k: usize, n: usize) -> Result<Self>",
    "description": "Create a new access controller on Gr(k,n)",
    "feature": "",
    "module": "controller"
  },
  {
    "name": "CapabilityIssuer",
    "kind": "struct",
    "signature": "CapabilityIssuer::generate() -> Self",
    "description": "Generate Ed25519 key pair for issuing signed capability tokens",
    "feature": "crypto",
    "module": "crypto"
  }
]
```

**Catalog coverage**: 29 entries across all 18 modules — controller, capability,
decision, composition, stability, audit, multi, rate_limit, routing, crdt, policy,
crypto, surreal_trust, verify, holographic, wasm, plus constants and cross-cutting types.

**LLM integration pattern**:
```
1. LLM calls `schubert discover` → receives JSON catalog
2. LLM identifies relevant types/functions from names and descriptions
3. LLM calls `schubert discover --feature <needed>` for context-specific details
4. LLM writes Rust code using the discovered API
```

---

### `recommend` — Configuration Recommender

Recommends optimal Schubert configuration given access control constraints.
Two modes: interactive (human Q&A) and batch (TOML file input for LLM automation).

#### Interactive Mode

```bash
schubert recommend
```

Walks through a series of questions:
```
=== Schubert Configuration Recommender ===

Number of user roles [3]: 5
Number of security domains (0 for single-domain) [0]: 2
Cross-domain capability translation? [y]: y
Audit logging? [y]: n
Cryptographic capability tokens? [n]: n
Policy-as-code (TOML files)? [y]: y
Trust model (discrete, continuous, surreal) [discrete]: continuous
Parallel batch operations? [n]: n
Formal verification (Karpal)? [n]: n
Holographic memory (Minuet)? [n]: n
WebAssembly deployment? [n]: n
Max concurrent principals [100]: 500
```

#### Batch Mode (LLM Automation)

```bash
schubert recommend --input constraints.toml
```

**TOML constraint file format**:
```toml
num_roles = 5
num_domains = 2
cross_domain = true
audit_required = true
crypto_required = false
trust_model = "surreal"
parallel_required = true
verification_required = false
holographic_required = false
policy_required = true
wasm_required = false
max_principals = 500
```

All fields are optional with sensible defaults:
| Field | Default | Notes |
|---|---|---|
| `num_roles` | 3 | Drives Grassmannian selection |
| `num_domains` | 0 | >2 triggers enterprise Gr(4,8) |
| `cross_domain` | false | Enables `MultiController` |
| `audit_required` | false | Requires `std` feature |
| `crypto_required` | false | Enables `crypto` feature |
| `trust_model` | `"discrete"` | `"continuous"` or `"surreal"` |
| `parallel_required` | false | Enables `parallel` + rayon |
| `verification_required` | false | Enables `karpal-verify` |
| `holographic_required` | false | Enables `holographic` + minuet |
| `policy_required` | false | Enables `policy` + toml |
| `wasm_required` | false | Enables `wasm` |
| `max_principals` | 100 | >1000 triggers Tropical path |

**Output** (JSON):
```json
{
  "grassmannian": [3, 6],
  "policy_dimension": 9,
  "computation_path": "Tropical (best for large-scale batch operations)",
  "features": ["std", "crypto", "surreal", "parallel", "policy", "karpal-verify"],
  "rationale": "Selected Gr(3,6) with dimension 9 for 5 roles across 2 domains. ...",
  "cargo_toml": "schubert = { version = \"0.1\", features = [\"std\", \"crypto\", ...] }",
  "example_code": "use schubert::AccessController;\n\nfn main() -> Result<...> {\n  ...\n}"
}
```

**LLM integration pattern**:
```
1. LLM analyzes user requirements → writes constraints.toml
2. LLM calls `schubert recommend --input constraints.toml`
3. LLM reads `cargo_toml` field → inserts into user's Cargo.toml
4. LLM reads `example_code` field → adapts for user's use case
```

**Grassmannian selection rules**:
| Constraint profile | Grassmannian | Dimension | Use case |
|---|---|---|---|
| ≤5 roles, 0-1 domains | Gr(2,4) | 4 | Standard RBAC |
| 5-8 roles, 1-2 domains | Gr(3,6) | 9 | Multi-tenant |
| >8 roles, >2 domains, cross-domain | Gr(4,8) | 16 | Enterprise |

---

### `explore` — Access Decision Sandbox

Evaluates hypothetical access decisions. Two modes: REPL (interactive exploration)
and one-shot (JSON evaluator for LLM tool-calling).

#### REPL Mode

```bash
schubert explore
```

```
Schubert Explorer v0.1.0
Type 'help' for commands, 'exit' to quit.

schubert> info
Schubert — quantitative access control via Schubert calculus
  Grassmannians: Gr(2,4), Gr(3,6), Gr(4,8)
  Decisions: Granted{n}, Impossible, Denied, Underconstrained
  Features: controller, composition, stability, audit, multi, routing,
            rate_limit, crdt, surreal_trust, crypto, verify, holographic

schubert> create 2 4
Created controller Gr(2,4) — dimension 4

schubert> stability alice
Stability analysis for 'alice':
  In a real session, this would call analyze_stability()
  Phase diagram shows trust breakpoints where capabilities become unstable.

schubert> compose alice read bob write
Composing alice::read with bob::write:
  Operadic composition checks if capabilities are geometrically compatible.
  Result includes multiplicity: how many configurations survive composition.

schubert> help
Commands:
  create <k> <n>          — Describe a Grassmannian Gr(k,n)
  info                     — Show Schubert overview
  stability <principal>    — Describe stability analysis
  compose <a> <cap> <b> <cap> — Describe composition
  help                     — Show this help
  exit | quit              — Exit the REPL
```

#### One-Shot Mode (LLM Tool-Calling)

```bash
schubert explore --eval '<json>'
```

**Supported actions**:

**`create`** — Describe a Grassmannian:
```bash
schubert explore --eval '{"action":"create","k":3,"n":6}'
```
```json
{
  "success": true,
  "message": "Created Gr(3,6) — Multi-tenant — dimension 9",
  "data": {
    "k": 3, "n": 6, "dimension": 9,
    "label": "Multi-tenant",
    "computation_paths": ["LR", "Localization", "Tropical", "Matroid"]
  }
}
```

**`grant`** — Describe a capability grant:
```bash
schubert explore --eval '{"action":"grant","principal":"alice","capability":"read","partition":[1],"kind":"ReadLike"}'
```
```json
{
  "success": true,
  "message": "Granted 'read' (partition [1], ReadLike) to alice",
  "data": {
    "principal": "alice", "capability": "read",
    "partition": [1], "kind": "ReadLike", "codimension": 1
  }
}
```

**`check`** — Describe an access check:
```bash
schubert explore --eval '{"action":"check","principal":"alice","capabilities":["read","write"]}'
```
```json
{
  "success": true,
  "message": "Check alice for [\"read\", \"write\"]",
  "data": {
    "principal": "alice", "capabilities": ["read", "write"],
    "possible_outcomes": [
      {"kind": "Granted", "configurations": "computed intersection count"},
      {"kind": "Impossible", "conflicting": "capabilities with zero intersection"},
      {"kind": "Denied", "reason": "overconstrained"},
      {"kind": "Underconstrained", "dimension": "remaining degrees of freedom"}
    ]
  }
}
```

**`stability`** — Describe stability analysis:
```bash
schubert explore --eval '{"action":"stability","principal":"alice"}'
```

**`compose`** — Describe operadic composition:
```bash
schubert explore --eval '{"action":"compose","a":"alice","cap_a":"read","b":"bob","cap_b":"write"}'
```

**`revoke`** — Describe capability revocation:
```bash
schubert explore --eval '{"action":"revoke","principal":"alice","capability":"read"}'
```

**`list`** — Describe the registry state:
```bash
schubert explore --eval '{"action":"list"}'
```

**LLM integration pattern**:
```
1. LLM receives user question: "Can alice read and write?"
2. LLM calls: schubert explore --eval '{"action":"check","principal":"alice","capabilities":["read","write"]}'
3. LLM reads the possible_outcomes and explains them to the user
4. LLM notes that "Impossible" means geometrically incompatible — stronger than boolean AND
```

**Error handling**:
```bash
schubert explore --eval '{"action":"create","k":0,"n":4}'
```
```json
{
  "success": false,
  "message": "Invalid Grassmannian: Gr(0,4) requires 0 < k < n"
}
```

```bash
schubert explore --eval 'not json'
```
```json
{
  "success": false,
  "message": "Invalid JSON command: ..."
}
```

---

## Feature Gate Awareness

Some CLI features require specific Cargo features. If a required feature is missing,
the CLI provides clear guidance:

```bash
# Without --features policy:
schubert recommend --input constraints.toml
# → Error: --input requires the 'policy' feature. Rebuild with --features policy
```

The `discover` catalog marks each entry with its required feature gate, so LLMs
can determine at discovery time which features they need:

```json
{
  "name": "CapabilityIssuer",
  "feature": "crypto",
  ...
}
```

---

## Integration Patterns

### LLM Agent Setup (One-Time)

```
1. schubert discover → learn full API surface
2. Analyze user's access control requirements
3. schubert recommend --input constraints.toml → get optimal config
4. Insert recommended dependencies into Cargo.toml
5. Adapt recommended example_code for user's use case
```

### LLM Runtime Tool-Calling

```
1. User asks: "Should we grant Alice access to read reports and write data?"
2. LLM calls: schubert explore --eval '{"action":"check","principal":"alice","capabilities":["read:reports","write:data"]}'
3. LLM interprets result: "Granted with 2 configurations" or "Impossible — these capabilities conflict geometrically"
```

### Human Exploration

```
1. schubert recommend → answer questions, get config with copy-paste Cargo.toml
2. schubert explore → interactively explore concepts
3. schubert discover --format md → browse API reference
```

---

## Exit Codes

| Code | Meaning |
|---|---|
| 0 | Success |
| 1 | Error (invalid input, missing features, I/O failure) |

---

## Limitations

- **The explore one-shot evaluator is descriptive, not operational.** It explains what
  *would* happen but doesn't run a real `AccessController`. For full evaluation, link
  against the `schubert` library directly.
- **The REPL is a teaching tool.** It demonstrates concepts interactively but doesn't
  maintain real controller state.
- **`recommend --input` requires the `policy` feature** (which includes `toml` parsing).
  Interactive mode works with default features.
