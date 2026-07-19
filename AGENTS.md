# Schubert — Agent Operating Map

> **Geometric access control via Schubert calculus.** Two artifacts ship from
> this repo: the **`schubert`** Rust crate (crates.io) and the
> **`@industrialalgebra/schubert-tsukoshi`** TypeScript package (npm), in
> `schubert-tsukoshi/`.

This is a project-level operating map. The global `~/.pi/agent/AGENTS.md` covers
**efficiency and model routing**; it does **not** carry this project's branch
discipline — that lives here. When the two could seem to compete (e.g. "be fast,
skip the PR"), the rules below win.

---

## Gitflow — HARD RULES (read first)

`develop` and `main` receive changes **only via merged PRs**. The full doctrine
is `/skill:ia-gitflow`; the non-negotiables:

1. **Never push directly to `develop` or `main`.** Branch `feature/*` (or
   `fix/*`) off `develop`, PR it, let CI run. Not "just a one-liner", not "last
   release tweak". *(v0.4.0 work was pushed straight to `develop`; CI never ran
   on it and `develop` went red, blocking the release.)*
2. **Every release to `main` is followed by a `main → develop` backmerge** using
   a **merge commit, never a squash**. The backmerge is the *last step of
   releasing*, not optional. *(v0.3.0's release was squash-merged with no
   backmerge; the divergence surfaced as a conflict at the v0.4.0 release PR.)*
3. **Release-only commits** (dating the CHANGELOG, a final version touch) live on
   a `release/v*` branch so they're reviewed — never pushed to `develop`.

Branch model: `feature/* → develop → release/v* → main → tag v* → publish.yml`.
Trunk-based repos (ia-toolkit, docs) skip `develop`; Rules 1–3 still apply to
`main`.

**Rebasing a PR** onto updated `develop`: `--force-with-lease` fails across the
GitHub+Forgejo dual-push remote ("stale info"). Use `git push --force` for a
deliberately rebased own branch; plain pushes hit both mirrors fine.

---

## Verification — run before claiming done

Don't assert "tests pass"; run them and read the output. The CI matrix:

```bash
cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test                       # default features
cargo test --features serde
cargo test --features karpal
cargo test --features parallel
cargo test --features serde,karpal,parallel
cargo test --all-features
cargo doc  --all-features --no-deps   # must show zero "unresolved link"
```

Notes: `cargo fmt --all` **does** format feature-gated files — never use
standalone `rustfmt --edition` (it disagrees with cargo-fmt's style). Every
feature-dependent example needs an explicit `[[example]] required-features = [...]`
in `Cargo.toml` or it breaks `cargo test` under default features. Zero clippy
warnings is a release gate (`-D warnings`).

**TypeScript** (`schubert-tsukoshi/`): `npm run build && npm test` (41 tests:
12 controller + 12 crypto + 17 CRDT). `dist/` is tracked, so a build isn't
required to publish — but run it as a verify.

Current Rust baseline: **193 tests** (159 lib + 18 CLI + 16 doc), zero clippy
across all six feature combinations.

---

## Read-first routing

Don't read whole large files cold. Grep, then targeted reads.

| Task | Read |
|---|---|
| Branch/release mechanics | this file → `/skill:ia-gitflow` |
| What a feature does | `src/<module>.rs` + `book/src/api/<module>.md` |
| Feature flags & combos | `Cargo.toml` `[features]` + `book/src/guide/feature-flags.md` |
| The math (σ, Grassmannians, LR) | `book/src/concepts/math.md`, `book/src/concepts/composition.md` |
| Crypto wire format (Rust↔TS) | `src/crypto.rs` doc-comments + `schubert-tsukoshi/src/crypto/wire.ts` |
| Multi-capability grants | `src/multi.rs` + `docs/handoff-multi-capability-tokens.md` |
| Release roadmap | `docs/ROADMAP.md` + `book/src/design/roadmap.md` |
| Versioning/CHANGELOG | `CHANGELOG.md`, `/skill:ia-version-bump` |
| Security posture | `book/src/design/threat-model.md`, `adversarial-concerns.md` |

---

## Release flow (condensed)

Full checklist: `/skill:ia-release-polish`. The shape:

1. **Version bump** (`/skill:ia-version-bump`) on a branch off `develop` → PR →
   `develop`. Leave `## [X.Y.Z] — Unreleased`.
2. **Cut `release/v<ver>`** off `develop`; **date the CHANGELOG** there; verify.
3. **Release PR `release/v<ver> → main`.** If it conflicts, a prior backmerge was
   skipped — see `/skill:ia-gitflow` "Reconciling a conflicting release PR"
   (confirm `develop` is a superset per-file, resolve metadata to the release
   branch, run the full matrix).
4. **Merge to `main`**; **tag `v<ver>`** on the merge commit; push the tag →
   `publish.yml` (`on: push: tags: ['v*']`) publishes the Rust crate to crates.io.
5. **`npm publish`** from `schubert-tsukoshi/` →
   `@industrialalgebra/schubert-tsukoshi@<ver>` (`publishConfig.access: public`
   is set; `dist/` is tracked).
6. **Backmerge `main → develop`** (Rule 2, merge commit) — closes the loop.
7. **Announce** via `/skill:ia-website`.

A version bump alone, or a merged release PR, is **not "shipped"**. Shipped =
published to crates.io + the tag exists + npm published + backmerge done.

---

## Feature flags

`std` (default) · `serde` · `crypto` (Ed25519 tokens, deps: ed25519-dalek, rand) ·
`axum` (bearer-token `AuthPrincipal` extractor; pulls `crypto` + axum + base64) ·
`karpal` (type-level composition proofs) · `parallel` · `holographic` (Minuet) ·
`surreal` (amari-surreal) · `karpal-verify`. Web-service combo: `axum` (which
enables `crypto`). Reference: `book/src/guide/feature-flags.md`.

---

## Conventions

- **License:** Apache-2.0 + CLA (since v0.2.0). See `/skill:ia-licensing`. Do not
  introduce AGPL deps — the network clause blocked enterprise adoption.
- **Coding standards:** TDD, phantom types, feature gates, exhaustive enums,
  composability. `/skill:ia-coding-standards`.
- **Toolchain:** nightly Rust (`rust-toolchain.toml`); rustfmt + clippy components.
- **Dependencies:** crates.io version deps (not path deps) for public release.
- **Mirror:** `origin` dual-pushes to GitHub + Forgejo (`king-ghidorah`). See
  `/skill:ia-forgejo-mirror`.
- **Efficiency/model routing:** covered by the global `AGENTS.md` — route
  reasoning to GLM, mechanical work to DeepSeek, one work unit per context.
