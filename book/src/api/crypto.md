# Proof-Carrying Tokens

Cryptographic capability tokens using Ed25519 signatures. Enable the `crypto` feature.

## Issuing Tokens

```rust
use schubert::crypto::{CapabilityIssuer, CapabilityToken};

// Generate a key pair
let issuer = CapabilityIssuer::generate();

// Issue a token
let token = issuer.issue("alice", "read:data")?;
let serialized = token.serialize()?; // send over the wire
```

## Verifying Tokens

```rust
use schubert::crypto::CapabilityVerifier;

let verifier = CapabilityVerifier::new(issuer.public_key());

// Verify and extract claims
verifier.verify(&token)?;
let (principal, capability) = verifier.verify_and_extract(&token)?;

// Grant in your controller
acl.grant(&principal, capability.as_str())?;
```

## Token Structure

```rust
pub struct CapabilityToken {
    pub principal: String,
    pub capability: String,
    pub issued_at: u64,
    pub expires_at: Option<u64>,
    pub signature: [u8; 64],
}
```

## Security Properties

- **Ed25519 signatures**: 128-bit security, 64-byte signatures
- **Tamper detection**: Modified tokens fail signature verification
- **Expiry support**: Tokens can include an expiration timestamp
- **No replay protection**: Tokens are stateless — the verifier tracks usage
- **Key rotation**: Generate new `CapabilityIssuer` and distribute public key
