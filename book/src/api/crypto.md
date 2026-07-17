# Proof-Carrying Tokens

Cryptographic capability tokens using Ed25519 signatures. Enable the `crypto`
feature:

```toml
[dependencies]
schubert = { version = "0.4", features = ["crypto"] }
```

Schubert ships two token kinds, both Ed25519-signed:

- **`CapabilityToken`** — a single capability, for the simple case.
- **`GrantToken`** — multiple capabilities, each carrying its Schubert partition,
  enabling geometric containment checks (write implies read, admin implies all)
  at verification time *without* a capability registry.

## Issuing Tokens

```rust
use schubert::crypto::CapabilityIssuer;

// Recommended: derive the issuer from a persisted 32-byte seed (see KeyStore).
let issuer = CapabilityIssuer::generate();

// Single-capability token
let token = issuer.issue("alice", "memory:read")?;

// Multi-capability grant — order-independent (canonicalized before signing)
use schubert::CapabilityId;
let grant = issuer.issue_grant("bob", &[
    (CapabilityId::new("memory:read"),  vec![1]),
    (CapabilityId::new("memory:write"), vec![2]),
])?;
```

Distribute the **public key** (`issuer.public_key()` / `issuer.public_key_hex()`)
to every verifier. The seed stays secret.

## Verifying Tokens

```rust
use schubert::crypto::{CapabilityVerifier, GrantVerifier};

// Single-capability tokens
let verifier = CapabilityVerifier::new(issuer.public_key());
verifier.verify(&token)?;                                   // signature check
let (principal, capability) = verifier.verify_and_extract(&token)?; // + claims

// Multi-capability grants
let grant_verifier = GrantVerifier::new(issuer.public_key());
grant_verifier.verify(&grant)?;                 // signature check
grant_verifier.may(&grant, &[1]);               // geometric containment (bool, see below)
```

`verify` reconstructs the canonical signing message and checks the Ed25519
signature with `verify_strict`. A token whose fields are altered after signing
fails verification.

## Geometric Containment (`GrantVerifier::may`)

The killer feature of grant tokens: a verifier can answer *does this grant
authorize capability P?* using only the signed partition data — no registry
lookup. `may(grant, required_partition)` returns `true` iff some granted
partition `λ` satisfies `required ≤ λ` component-wise:

```rust
// A grant carrying only `write` (partition [2]):
assert!( grant_verifier.may(&grant, &[1]) );       // read  — [1] ≤ [2]
assert!( grant_verifier.may(&grant, &[2]) );       // write — explicit
assert!(!grant_verifier.may(&grant, &[2, 1]) );    // manage — not implied

// A grant carrying `admin` (partition [4,4,4,4] on Gr(4,8)) implies everything:
assert!( admin_grant_verifier.may(&admin_grant, &[1]) );
assert!( admin_grant_verifier.may(&admin_grant, &[2, 1]) );
```

No special-casing — "write implies read" and "admin implies all" fall out of
the partition lattice order.

## Wire Format

Both tokens serialize to a length-prefixed binary blob via associated
`to_bytes` / `from_bytes` functions, suitable for base64-encoding as a bearer
token:

```rust
use schubert::crypto::{CapabilityToken, GrantToken};

let bytes = CapabilityToken::to_bytes(&token);   // Vec<u8>
let roundtrip = CapabilityToken::from_bytes(&bytes)?;  // Result<_>

let g_bytes = GrantToken::to_bytes(&grant);
```

**`CapabilityToken`** layout: `u16 BE principal_len | principal | u16 BE
capability_len | capability | 32B issuer key | 64B signature`.

**`GrantToken`** layout: `u16 BE principal_len | principal | u16 BE cap_count |
per cap: u16 BE id_len | id | u8 partition_len | partition bytes | 32B issuer
key | 64B signature`.

> The TypeScript extraction (`schubert-tsukoshi`) uses this exact wire format —
> tokens issued in Rust verify in TS and vice-versa.

## Key Persistence (`KeyStore`)

Persist an issuer identity across restarts by storing its 32-byte seed. On Unix
the file is created mode `0600` (owner read/write only):

```rust
use schubert::crypto::KeyStore;

// Load an existing seed, or create one if absent.
let seed = KeyStore::load_or_create(std::path::Path::new("/var/lib/app/issuer.key"))?;
let issuer = CapabilityIssuer::from_seed(seed);

// Or read-only:
let seed = KeyStore::load(std::path::Path::new("issuer.key"))?;
```

`load_or_create` is atomic against concurrent startup (`create_new`); it fails
closed rather than clobbering an existing key.

## Token Structures

```rust
pub struct CapabilityToken {
    pub principal: PrincipalId,
    pub capability: CapabilityId,
    pub issuer_key: Vec<u8>,   // 32 bytes
    pub signature: Vec<u8>,   // 64 bytes
}

pub struct GrantToken {
    pub principal: PrincipalId,
    pub capabilities: Vec<GrantCapability>,
    pub issuer_key: Vec<u8>,
    pub signature: Vec<u8>,
}

pub struct GrantCapability {
    pub id: CapabilityId,
    pub partition: Vec<usize>,
}
```

## Security Properties

- **Ed25519 signatures** — 128-bit security, 64-byte detached signatures,
  verified with `verify_strict` (rejects malleable signatures).
- **Issuer key bound into the message** — prevents key substitution: a token
  cannot be re-credited to a different issuer.
- **Tamper detection** — any field changed after signing fails verification.
- **Order-independent grants** — capabilities are canonically sorted before
  signing, so grant construction order does not affect the signature.
- **No replay protection** — tokens are stateless. For one-time use, track
  consumed token nonces server-side, or pair with short-lived expiry windows.
- **Key rotation** — generate a new `CapabilityIssuer` and distribute its public
  key; old tokens continue to verify against their original issuer's key.

> **Partition components are stored as `u8`** in the grant signing message, so
> partitions are limited to parts ≤ 255 (ample for any realistic Grassmannian;
> Gr(4,8)'s largest part is 4).
