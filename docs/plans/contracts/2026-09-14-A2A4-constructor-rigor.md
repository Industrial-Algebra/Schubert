# Plan Contract — v0.6.0 Workstream A, Unit A2+A4: ID validation + verifier constructor rigor

**Repo:** Schubert · **Branch:** `feature/a2-a4-constructor-rigor` (already checked out)
**Depends on:** sprint plan PR #55 (merged) · **Followed by:** A3 (partition validation at issuance)
**Source:** RABBIT_HOLE_2026-09-12_Schubert-CryptoLayer.md §2.3, §2.5, §3 items (2) and (4)

## Goal

1. **A2** — `PrincipalId::new` and `CapabilityId::new` validate and return
   `Result`, rejecting empty strings and strings containing a NUL byte. The
   bypass paths (`From<&str>`, `From<String>`) are **removed** so no
   construction route skips validation. All `impl Into<PrincipalId>` /
   `impl Into<CapabilityId>` parameters become by-value `PrincipalId` /
   `CapabilityId`.
2. **A4** — `CapabilityVerifier::new` and `GrantVerifier::new` return
   `Result` instead of panicking on malformed keys.

**TDD: write every test in the Tests section FIRST, confirm it fails to
compile or fails red, then implement.**

## Files

Modify: `src/principal.rs`, `src/capability.rs`, `src/error.rs`,
`src/crypto.rs`, plus every file matched by the sweep greps below (call-site
updates only — mechanical, compiler-checked).

Files out of bounds: `schubert-tsukoshi/**`, `CHANGELOG.md`, `Cargo.toml`,
`docs/**`, `.github/**`, `benches/**` (benches ARE in bounds — they contain
call sites; the fence is: do not add new benchmarks).

## Contracts

### C1 — `src/principal.rs`: validating constructor (replaces lines 38-40)

```rust
    /// Create a validated principal identifier.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::SchubertError::InvalidPrincipalId`] if `id`
    /// is empty or contains a NUL byte.
    pub fn new(id: impl Into<String>) -> crate::error::Result<Self> {
        let id = id.into();
        if id.is_empty() || id.contains('\0') {
            return Err(crate::error::SchubertError::InvalidPrincipalId(id));
        }
        Ok(Self(id))
    }
```

### C2 — `src/principal.rs`: delete the bypass From impls

Delete both blocks (currently lines 53-61):

```rust
impl From<&str> for PrincipalId { ... }
impl From<String> for PrincipalId { ... }
```

### C3 — `src/principal.rs`: `Principal::new` takes `PrincipalId` by value

Change the signature from `impl Into<PrincipalId>` to the concrete type and
drop the `.into()`:

```rust
    pub fn new(id: PrincipalId, k: usize, n: usize) -> crate::error::Result<Self> {
        let namespace = NamespaceBuilder::new(id.as_str(), k, n)
            .build()
            .map_err(crate::error::SchubertError::Enumerative)?;
        Ok(Self {
            id,
            namespace,
            granted_capability_ids: Vec::new(),
            created_at: now_millis(),
        })
    }
```

(The body above is the full final body; everything after `Principal::new`
in the file is unchanged.)

### C4 — `src/capability.rs`: mirror C1-C3

- `CapabilityId::new` becomes the validating `Result` constructor (same
  shape as C1, error variant `InvalidCapabilityId`).
- Delete `impl From<&str> for CapabilityId` and `impl From<String> for
  CapabilityId` (currently lines 59-67).
- `Capability::new` first parameter changes from
  `id: impl Into<CapabilityId>` to `id: CapabilityId`; body field init
  `id: id.into()` becomes `id`.

### C5 — `src/error.rs`: three new variants (insert near `GrantExpired`)

```rust
    /// A principal identifier failed validation (empty, or contains a NUL
    /// byte that would make the NUL-terminated signing message ambiguous).
    InvalidPrincipalId(String),

    /// A capability identifier failed validation (empty, or contains a NUL
    /// byte that would make the NUL-terminated signing message ambiguous).
    InvalidCapabilityId(String),

    /// A verifying key was malformed (wrong length or invalid Ed25519 bytes).
    InvalidVerifyingKey(String),
```

Each with a `#[error("...")]` attribute in the file's existing style:
`invalid principal id: {0:?}`, `invalid capability id: {0:?}`,
`invalid verifying key: {0}`.

### C6 — `src/crypto.rs`: both verifier constructors return `Result`

Apply to **both** `CapabilityVerifier::new` (line ~312) and
`GrantVerifier::new` (line ~625) — identical body:

```rust
    /// Create a verifier from the issuer's public key bytes.
    ///
    /// # Errors
    ///
    /// Returns [`SchubertError::InvalidVerifyingKey`] if `public_key` is
    /// not exactly 32 bytes or is not a valid Ed25519 verifying key.
    pub fn new(public_key: Vec<u8>) -> Result<Self> {
        let bytes: [u8; 32] = public_key
            .as_slice()
            .try_into()
            .map_err(|_| SchubertError::InvalidVerifyingKey(format!(
                "expected 32 bytes, got {}",
                public_key.len()
            )))?;
        let verifying_key = VerifyingKey::from_bytes(&bytes).map_err(|_| {
            SchubertError::InvalidVerifyingKey("invalid Ed25519 public key".into())
        })?;
        Ok(Self { verifying_key })
    }
```

(`Result` is already imported at crypto.rs line 35.)

### C7 — `src/crypto.rs`: `impl Into<...>` parameters become by-value

Every parameter `principal: impl Into<PrincipalId>` becomes
`principal: PrincipalId`; every `capability: impl Into<CapabilityId>`
becomes `capability: CapabilityId`; every
`principal: impl Into<PrincipalId> + Clone` becomes
`principal: PrincipalId` (drop the `+ Clone`). Sites (from grep, ~lines
107-108, 171-172, 200-201, 226, 239, 256): the `CapabilityToken` issue
constructor, `issue_grant`, `issue_grant_with_expiry`,
`issue_grant_with_options`, `issue_grant_under_policy`, and any bearer
helper. Interior constructions then read
`principal: PrincipalId::new(principal)?` (these functions already return
`Result`). **Exception:** if a site's parameter is moved into a struct
field without string conversion (wire decode at lines ~575, ~604), keep
`PrincipalId::new(x)?` there too — decode now propagates the new error.

## Sweep (compiler-checked call-site updates)

Run both greps; every hit must be updated:

```sh
grep -rln 'PrincipalId::new\|CapabilityId::new\|Principal::new(\|Capability::new(' src examples benches tests book/src 2>/dev/null
grep -rn 'Into<PrincipalId>\|Into<CapabilityId>' src examples benches tests book/src 2>/dev/null
```

Rules:

| Context | Before | After |
|---|---|---|
| tests, examples, benches, doc-tests | `PrincipalId::new("x")` | `PrincipalId::new("x").expect("valid id")` |
| tests, examples, benches, doc-tests | `Principal::new("x", k, n)` | `Principal::new(PrincipalId::new("x").expect("valid id"), k, n)` |
| same, capability | `Capability::new("x", ...)` | `Capability::new(CapabilityId::new("x").expect("valid id"), ...)` |
| lib fns returning `Result` | construction | propagate with `?` |
| verifier construction in tests/examples | `GrantVerifier::new(k)` | `GrantVerifier::new(k).expect("valid key")` (same for `CapabilityVerifier`) |

The compiler is the checklist: after the sweep, zero errors and zero
warnings under `-D warnings` (both clippy configurations in Completion).

## Tests (write first; every assertion enumerated)

New tests (add to the existing `#[cfg(test)]` module of each file):

1. `principal_id_new_rejects_empty` —
   `assert!(PrincipalId::new("").is_err());` and
   `assert!(matches!(PrincipalId::new(""), Err(SchubertError::InvalidPrincipalId(_))));`
2. `principal_id_new_rejects_interior_nul` — same two assertions with
   `"a\0b"`.
3. `principal_id_new_accepts_valid` —
   `let p = PrincipalId::new("alice").expect("valid");`
   `assert_eq!(p.as_str(), "alice");`
4. `capability_id_new_rejects_empty` — mirror of 1 with
   `InvalidCapabilityId`.
5. `capability_id_new_rejects_interior_nul` — mirror of 2.
6. `capability_id_new_accepts_valid` — `CapabilityId::new("read")` →
   `as_str() == "read"`.
7. `grant_verifier_rejects_wrong_length_key` (crypto feature) —
   `assert!(matches!(GrantVerifier::new(vec![0u8; 31]), Err(SchubertError::InvalidVerifyingKey(_))));`
   and the same for `vec![0u8; 33]`.
8. `grant_verifier_accepts_32_byte_key` (crypto) — generate a
   `SigningKey`, take `verifying_key.as_bytes().to_vec()`, assert
   `GrantVerifier::new(bytes).is_ok()`.
9. `capability_verifier_rejects_wrong_length_key` (crypto) — mirror of 7.
10. `capability_verifier_accepts_32_byte_key` (crypto) — mirror of 8.
11. `grant_wire_decode_rejects_embedded_nul_principal` (crypto) — issue a
    grant for a valid principal, encode it, splice a NUL into the
    principal bytes (read the wire layout in crypto.rs docs to locate:
    `u16BE len | principal | ...` — the principal is the first field),
    assert `matches!(GrantToken::decode(&bytes), Err(SchubertError::InvalidPrincipalId(_)))`.
    Signature tampering is irrelevant here: decode must reject on the
    identifier before any signature check.
12. `verifier_constructors_do_not_panic_on_short_input` (crypto) — call
    both constructors with `vec![0u8; 8]`; assert `is_err()` (this test
    fails red today because the current code panics).

Feature-gating: crypto tests live in the existing crypto-gated test module
(`#[cfg(feature = "crypto")]`); principal/capability tests are
unconditional. Mentally compile the no-crypto build: nothing outside
crypto feature code may reference `InvalidVerifyingKey` or the verifiers.

All existing tests must still pass unchanged (call sites updated per the
sweep rules only — do not weaken any existing assertion).

## Constraints

- No new crates; `thiserror = "2"` stays.
- License headers untouched; no reformatting of untouched regions.
- `cargo fmt --all` (cargo's fmt, never standalone rustfmt).
- Serde deserialization of `PrincipalId`/`CapabilityId` is a **known,
  out-of-scope bypass** (the wire format is the validated boundary) — do
  not touch serde impls; note it in your report.

## Completion (pipe through the filters; all must be green / zero)

```sh
cargo test 2>&1 | grep -E 'test result|FAILED|^error' | tail -15
cargo test --all-features 2>&1 | grep -E 'test result|FAILED' | tail -8
cargo clippy --all-targets -- -D warnings 2>&1 | tail -4
cargo clippy --all-targets --all-features -- -D warnings 2>&1 | tail -4
cargo fmt --all --check && echo FMT-OK
cargo doc --all-features --no-deps 2>&1 | grep -c "unresolved link"
```

## Out of scope

- A3 (issuance partition validation), A1 (`VerifiedGrant`), A5 (audit),
  A6 (tsukoshi) — do not implement any of them.
- Wire format bytes, signing message layout, CHANGELOG, docs book prose
  beyond code fences swept above.
- Any `unwrap`/`expect` in library non-test code paths.
