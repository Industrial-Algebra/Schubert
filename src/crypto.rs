// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! Cryptographic capability tokens for distributed access control.
//!
//! Proof-carrying capabilities as Ed25519-signed tokens. A capability
//! issuer signs a token containing a capability definition and a principal
//! identifier. Verifiers check the signature and the Schubert intersection
//! to authorize access — no shared database required.
//!
//! # Architecture
//!
//! ```text
//! Issuer                         Verifier
//! ──────                         ────────
//! signing_key ──→ sign(cap) ──→  token ──→ verify(sig, pubkey)
//!                                            └──→ check(cap, principal)
//! ```
//!
//! # Example
//!
//! ```
//! # #[cfg(feature = "crypto")] {
//! use schubert::crypto::{CapabilityIssuer, CapabilityVerifier, CapabilityToken};
//!
//! let issuer = CapabilityIssuer::generate();
//! let token = issuer.issue("alice", "read:data")?;
//!
//! let verifier = CapabilityVerifier::new(issuer.public_key());
//! assert!(verifier.verify(&token).is_ok());
//! # }
//! # Ok::<(), schubert::SchubertError>(())
//! ```

use crate::{CapabilityId, PrincipalId, Result, SchubertError};
use ed25519_dalek::{
    Signature, Signer, SigningKey, VerifyingKey, PUBLIC_KEY_LENGTH, SIGNATURE_LENGTH,
};
use rand::rngs::OsRng;

const ISSUER_KEY_LEN: usize = PUBLIC_KEY_LENGTH;
const SIG_LEN: usize = SIGNATURE_LENGTH;

/// A cryptographic capability token — a signed assertion that a principal
/// holds a specific capability.
///
/// Tokens are self-contained: they carry the principal ID, capability ID,
/// public key, and Ed25519 signature. Verifiers need only the issuer's
/// public key to validate.
#[derive(Debug, Clone)]
pub struct CapabilityToken {
    /// The principal this token is issued to.
    pub principal: PrincipalId,
    /// The capability granted by this token.
    pub capability: CapabilityId,
    /// The issuer's public key (Ed25519, 32 bytes).
    pub issuer_key: Vec<u8>,
    /// Ed25519 signature over `principal || capability || issuer_key`.
    pub signature: Vec<u8>,
}

impl CapabilityToken {
    /// Serialize the token to a binary wire format.
    ///
    /// Produces a length-prefixed binary blob:
    /// ```text
    /// u16 BE principal_len | principal utf-8
    /// u16 BE capability_len | capability utf-8
    /// 32 bytes issuer public key
    /// 64 bytes Ed25519 signature
    /// ```
    ///
    /// Callers typically base64-encode the result for bearer token use.
    pub fn to_bytes(token: &Self) -> Vec<u8> {
        let p = token.principal.as_str().as_bytes();
        let c = token.capability.as_str().as_bytes();
        let mut buf = Vec::with_capacity(2 + p.len() + 2 + c.len() + ISSUER_KEY_LEN + SIG_LEN);
        buf.extend_from_slice(&(p.len() as u16).to_be_bytes());
        buf.extend_from_slice(p);
        buf.extend_from_slice(&(c.len() as u16).to_be_bytes());
        buf.extend_from_slice(c);
        buf.extend_from_slice(&token.issuer_key);
        buf.extend_from_slice(&token.signature);
        buf
    }

    /// Deserialize a token from the binary wire format.
    ///
    /// # Errors
    ///
    /// Returns an error if the byte slice is truncated or malformed.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let mut pos = 0;
        let plen = read_u16(bytes, &mut pos)?;
        let principal = read_str(bytes, &mut pos, plen)?;
        let clen = read_u16(bytes, &mut pos)?;
        let capability = read_str(bytes, &mut pos, clen)?;
        let issuer_key = read_bytes(bytes, &mut pos, ISSUER_KEY_LEN)?;
        let signature = read_bytes(bytes, &mut pos, SIG_LEN)?;
        if pos != bytes.len() {
            return Err(SchubertError::CryptoVerificationFailed(
                "trailing bytes in token".into(),
            ));
        }
        Ok(CapabilityToken {
            principal: PrincipalId::new(principal),
            capability: CapabilityId::new(capability),
            issuer_key: issuer_key.to_vec(),
            signature: signature.to_vec(),
        })
    }
}

/// An issuer of cryptographic capability tokens.
///
/// Holds an Ed25519 signing key and can issue tokens to principals.
/// The corresponding public key is distributed to verifiers.
#[derive(Debug)]
pub struct CapabilityIssuer {
    signing_key: SigningKey,
}

impl CapabilityIssuer {
    /// Generate a new issuer with a random key pair.
    pub fn generate() -> Self {
        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        Self { signing_key }
    }

    /// Create an issuer from an existing signing key.
    pub fn from_key(signing_key: SigningKey) -> Self {
        Self { signing_key }
    }

    /// Create an issuer from a 32-byte Ed25519 seed.
    ///
    /// This is the recommended way to persist and restore an issuer
    /// identity across process restarts. The seed is the raw Ed25519
    /// key material — store it securely (e.g. in a 0600 file).
    pub fn from_seed(seed: [u8; 32]) -> Self {
        let signing_key = SigningKey::from_bytes(&seed);
        Self { signing_key }
    }

    /// Return the public key for distribution to verifiers.
    pub fn public_key(&self) -> Vec<u8> {
        self.signing_key.verifying_key().to_bytes().to_vec()
    }

    /// Return the public key as a lowercase hex string.
    ///
    /// Produces a 64-character hex string suitable for display,
    /// configuration files, and operator visibility.
    pub fn public_key_hex(&self) -> String {
        self.signing_key
            .verifying_key()
            .to_bytes()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect()
    }

    /// Issue a capability token to a principal.
    ///
    /// Signs the tuple `(principal_id, capability_id, public_key)` with
    /// the issuer's Ed25519 key.
    pub fn issue(
        &self,
        principal: impl Into<PrincipalId>,
        capability: impl Into<CapabilityId>,
    ) -> Result<CapabilityToken> {
        let principal = principal.into();
        let capability = capability.into();
        let key_bytes = self.signing_key.verifying_key().to_bytes();

        // Build message: principal || capability || public_key
        let mut message = Vec::new();
        message.extend_from_slice(principal.as_str().as_bytes());
        message.push(0); // separator
        message.extend_from_slice(capability.as_str().as_bytes());
        message.push(0);
        message.extend_from_slice(&key_bytes);

        let signature = self.signing_key.sign(&message);

        Ok(CapabilityToken {
            principal,
            capability,
            issuer_key: key_bytes.to_vec(),
            signature: signature.to_bytes().to_vec(),
        })
    }

    /// Issue multiple tokens at once.
    pub fn issue_batch(
        &self,
        grants: &[(
            impl Into<PrincipalId> + Clone,
            impl Into<CapabilityId> + Clone,
        )],
    ) -> Result<Vec<CapabilityToken>> {
        grants
            .iter()
            .map(|(p, c)| self.issue(p.clone(), c.clone()))
            .collect()
    }

    /// Issue a grant token carrying multiple capabilities.
    ///
    /// Each entry in `capabilities` is a `(capability_id, partition)` pair.
    /// The partition is signed alongside the ID, enabling geometric
    /// containment checks (write-implies-read, admin-implies-all) at
    /// verification time without access to a capability registry.
    ///
    /// Capabilities are canonically sorted by partition (component-wise)
    /// before signing, so order-independent grants produce the same signature.
    pub fn issue_grant(
        &self,
        principal: impl Into<PrincipalId>,
        capabilities: &[(CapabilityId, Vec<usize>)],
    ) -> Result<GrantToken> {
        let principal = principal.into();
        let key_bytes = self.signing_key.verifying_key().to_bytes();

        let mut entries: Vec<GrantCapability> = capabilities
            .iter()
            .map(|(id, partition)| GrantCapability {
                id: id.clone(),
                partition: partition.clone(),
            })
            .collect();

        // Canonical sort: by component-wise partition comparison, then by ID.
        entries.sort_by(|a, b| {
            a.partition
                .cmp(&b.partition)
                .then_with(|| a.id.as_str().cmp(b.id.as_str()))
        });

        // Build signing message: principal || canonical_entries || issuer_key
        let mut message = Vec::new();
        message.extend_from_slice(principal.as_str().as_bytes());
        message.push(0);
        for entry in &entries {
            message.extend_from_slice(entry.id.as_str().as_bytes());
            message.push(0);
            // Encode partition: u8 count, then u8 per part
            message.push(entry.partition.len() as u8);
            for &part in &entry.partition {
                message.push(part as u8);
            }
        }
        message.push(0);
        message.extend_from_slice(&key_bytes);

        let signature = self.signing_key.sign(&message);

        Ok(GrantToken {
            principal,
            capabilities: entries,
            issuer_key: key_bytes.to_vec(),
            signature: signature.to_bytes().to_vec(),
        })
    }
}

/// A verifier of cryptographic capability tokens.
///
/// Holds the issuer's public key and validates token signatures.
/// Can be combined with an [`AccessController`](crate::AccessController)
/// for full geometric access checking.
#[derive(Debug)]
pub struct CapabilityVerifier {
    verifying_key: VerifyingKey,
}

impl CapabilityVerifier {
    /// Create a verifier from the issuer's public key bytes.
    pub fn new(public_key: Vec<u8>) -> Self {
        let bytes: [u8; 32] = public_key[..32]
            .try_into()
            .expect("public key must be 32 bytes");
        let verifying_key = VerifyingKey::from_bytes(&bytes).expect("invalid Ed25519 public key");
        Self { verifying_key }
    }

    /// Verify a token's signature.
    ///
    /// Returns `Ok(())` if the signature is valid for the claimed
    /// principal and capability under this issuer's public key.
    pub fn verify(&self, token: &CapabilityToken) -> Result<()> {
        // Reconstruct the signed message
        let mut message = Vec::new();
        message.extend_from_slice(token.principal.as_str().as_bytes());
        message.push(0);
        message.extend_from_slice(token.capability.as_str().as_bytes());
        message.push(0);
        message.extend_from_slice(&token.issuer_key);

        // Verify the issuer key matches
        if token.issuer_key != self.verifying_key.to_bytes() {
            return Err(SchubertError::CryptoVerificationFailed(
                "token issuer key does not match verifier key".into(),
            ));
        }

        // Verify signature
        let sig_bytes: [u8; 64] = token.signature.as_slice().try_into().map_err(|_| {
            SchubertError::CryptoVerificationFailed("invalid signature length".into())
        })?;
        let signature = Signature::from_bytes(&sig_bytes);

        self.verifying_key
            .verify_strict(&message, &signature)
            .map_err(|e| SchubertError::CryptoVerificationFailed(format!("bad signature: {e}")))?;

        Ok(())
    }

    /// Verify a token and extract the principal and capability IDs.
    ///
    /// Returns the `(principal_id, capability_id)` if valid.
    pub fn verify_and_extract<'a>(
        &self,
        token: &'a CapabilityToken,
    ) -> Result<(&'a PrincipalId, &'a CapabilityId)> {
        self.verify(token)?;
        Ok((&token.principal, &token.capability))
    }

    /// Verify multiple tokens in parallel.
    #[cfg(feature = "parallel")]
    pub fn verify_batch(&self, tokens: &[CapabilityToken]) -> Vec<Result<()>> {
        use rayon::prelude::*;
        tokens.par_iter().map(|t| self.verify(t)).collect()
    }
}

// ---------------------------------------------------------------------------
// Grant types — multi-capability tokens with geometric containment
// ---------------------------------------------------------------------------

/// A capability entry within a grant token.
///
/// Carries both the capability ID and its Schubert partition.
/// Both fields are cryptographically signed — tampering with either
/// invalidates the token.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GrantCapability {
    /// The capability identifier.
    pub id: CapabilityId,
    /// The Schubert partition defining this capability's geometric condition.
    pub partition: Vec<usize>,
}

/// A multi-capability grant token.
///
/// A grant carries a principal, a set of capabilities (each with its
/// partition), the issuer's public key, and an Ed25519 signature.
/// This supersedes `CapabilityToken` for consumers that need multiple
/// capabilities in a single signed token (Ijima, Dominic).
///
/// Singleton grants (`[cap]`) are operationally equivalent to single-cap
/// `CapabilityToken`s. The partition data enables geometric containment
/// checks (write-implies-read, admin-implies-all) at verification time
/// without access to a capability registry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GrantToken {
    /// The principal this grant is issued to.
    pub principal: PrincipalId,
    /// The granted capabilities with their partitions.
    pub capabilities: Vec<GrantCapability>,
    /// The issuer's public key (Ed25519, 32 bytes).
    pub issuer_key: Vec<u8>,
    /// Ed25519 signature over the canonical encoding of principal,
    /// capabilities, and issuer key.
    pub signature: Vec<u8>,
}

impl GrantToken {
    /// Serialize a grant token to the binary wire format.
    ///
    /// ```text
    /// u16 BE principal_len | principal utf-8
    /// u16 BE cap_count
    /// for each cap:
    ///   u16 BE cap_id_len | cap_id utf-8
    ///   u8 partition_len | partition bytes
    /// 32 bytes issuer public key
    /// 64 bytes Ed25519 signature
    /// ```
    pub fn to_bytes(token: &Self) -> Vec<u8> {
        let mut buf = Vec::new();
        let p = token.principal.as_str().as_bytes();
        buf.extend_from_slice(&(p.len() as u16).to_be_bytes());
        buf.extend_from_slice(p);
        buf.extend_from_slice(&(token.capabilities.len() as u16).to_be_bytes());
        for cap in &token.capabilities {
            let c = cap.id.as_str().as_bytes();
            buf.extend_from_slice(&(c.len() as u16).to_be_bytes());
            buf.extend_from_slice(c);
            buf.push(cap.partition.len() as u8);
            for &part in &cap.partition {
                buf.push(part as u8);
            }
        }
        buf.extend_from_slice(&token.issuer_key);
        buf.extend_from_slice(&token.signature);
        buf
    }

    /// Deserialize a grant token from the binary wire format.
    ///
    /// # Errors
    ///
    /// Returns an error if the byte slice is truncated or malformed.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let mut pos = 0;
        let plen = read_u16(bytes, &mut pos)?;
        let principal = read_str(bytes, &mut pos, plen)?;
        let cap_count = read_u16(bytes, &mut pos)?;
        let mut capabilities = Vec::with_capacity(cap_count);
        for _ in 0..cap_count {
            let clen = read_u16(bytes, &mut pos)?;
            let cap_id = read_str(bytes, &mut pos, clen)?;
            let plen = read_u8(bytes, &mut pos)?;
            let partition: Vec<usize> = (0..plen)
                .map(|_| read_u8(bytes, &mut pos))
                .collect::<std::result::Result<_, _>>()
                .map_err(|_| {
                    SchubertError::CryptoVerificationFailed(
                        "truncated partition in grant token".into(),
                    )
                })?;
            capabilities.push(GrantCapability {
                id: CapabilityId::new(cap_id),
                partition,
            });
        }
        let issuer_key = read_bytes(bytes, &mut pos, ISSUER_KEY_LEN)?;
        let signature = read_bytes(bytes, &mut pos, SIG_LEN)?;
        if pos != bytes.len() {
            return Err(SchubertError::CryptoVerificationFailed(
                "trailing bytes in grant token".into(),
            ));
        }
        Ok(GrantToken {
            principal: PrincipalId::new(principal),
            capabilities,
            issuer_key: issuer_key.to_vec(),
            signature: signature.to_vec(),
        })
    }
}

/// Verifier for multi-capability grant tokens.
///
/// Checks cryptographic signatures and performs geometric containment
/// checks using the signed partition data carried in the grant.
#[derive(Debug)]
pub struct GrantVerifier {
    verifying_key: VerifyingKey,
}

impl GrantVerifier {
    /// Create a verifier from the issuer's public key bytes.
    pub fn new(public_key: Vec<u8>) -> Self {
        let bytes: [u8; 32] = public_key[..32]
            .try_into()
            .expect("public key must be 32 bytes");
        let verifying_key = VerifyingKey::from_bytes(&bytes).expect("invalid Ed25519 public key");
        Self { verifying_key }
    }

    /// Verify a grant token's Ed25519 signature.
    ///
    /// Reconstructs the signing message from the token's fields and
    /// checks the signature against the issuer's public key.
    pub fn verify(&self, grant: &GrantToken) -> Result<()> {
        // Verify the issuer key matches
        if grant.issuer_key != self.verifying_key.to_bytes() {
            return Err(SchubertError::CryptoVerificationFailed(
                "grant issuer key does not match verifier key".into(),
            ));
        }

        // Reconstruct signing message (must match issue_grant exactly)
        let mut message = Vec::new();
        message.extend_from_slice(grant.principal.as_str().as_bytes());
        message.push(0);
        for cap in &grant.capabilities {
            message.extend_from_slice(cap.id.as_str().as_bytes());
            message.push(0);
            message.push(cap.partition.len() as u8);
            for &part in &cap.partition {
                message.push(part as u8);
            }
        }
        message.push(0);
        message.extend_from_slice(&grant.issuer_key);

        let sig_bytes: [u8; 64] = grant.signature.as_slice().try_into().map_err(|_| {
            SchubertError::CryptoVerificationFailed("invalid grant signature length".into())
        })?;
        let signature = Signature::from_bytes(&sig_bytes);

        self.verifying_key
            .verify_strict(&message, &signature)
            .map_err(|e| {
                SchubertError::CryptoVerificationFailed(format!("bad grant signature: {e}"))
            })?;

        Ok(())
    }

    /// Check geometric containment: does the grant imply the given capability?
    ///
    /// Returns `true` iff any granted partition `λ` satisfies
    /// `cap_partition ≤ λ` component-wise. This is geometric containment
    /// via the Schubert variety partition lattice — not set membership.
    ///
    /// # Properties
    ///
    /// - **Write implies read:** If `[2]` is granted, `[1]` is implied
    ///   (because `[1] ≤ [2]` component-wise).
    /// - **Admin implies all:** If `[4,4,4,4]` is granted on Gr(4,8), every
    ///   partition is implied (because all partitions are ≤ the max).
    pub fn may(&self, grant: &GrantToken, cap_partition: &[usize]) -> bool {
        grant
            .capabilities
            .iter()
            .any(|cap| partitions_le(cap_partition, &cap.partition))
    }
}

// ---------- partition helpers ----------

/// Returns `true` if `a ≤ b` component-wise, padding shorter sequences with zeros.
fn partitions_le(a: &[usize], b: &[usize]) -> bool {
    let max_len = a.len().max(b.len());
    for i in 0..max_len {
        let av = a.get(i).copied().unwrap_or(0);
        let bv = b.get(i).copied().unwrap_or(0);
        if av > bv {
            return false;
        }
    }
    true
}

// ---------------------------------------------------------------------------
// Key persistence — #16.5
// ---------------------------------------------------------------------------

/// File-based Ed25519 seed persistence.
///
/// Provides load-or-create semantics for an issuer key seed.
/// The seed is stored as a raw 32-byte file with platform-appropriate
/// permissions (mode `0600` on Unix).
///
/// # Example
///
/// ```
/// # use std::path::Path;
/// # use schubert::crypto::KeyStore;
/// # let dir = std::env::temp_dir().join(format!("schubert-keystore-test-{}", std::process::id()));
/// # std::fs::create_dir_all(&dir).unwrap();
/// let path = dir.join("issuer.key");
///
/// // First call creates the file with a random seed.
/// let seed = KeyStore::load_or_create(&path).unwrap();
///
/// // Second call reads the same seed back.
/// let same_seed = KeyStore::load_or_create(&path).unwrap();
/// assert_eq!(seed, same_seed);
///
/// // Direct load also works.
/// let loaded = KeyStore::load(&path).unwrap();
/// assert_eq!(seed, loaded);
/// ```
pub struct KeyStore;

impl KeyStore {
    /// Load an existing 32-byte key seed from `path`, or create it with a
    /// fresh random value if the file does not exist.
    ///
    /// On Unix, creates the file with mode `0600` (owner read/write only).
    /// Creates parent directories as needed.
    ///
    /// # Errors
    ///
    /// Returns an I/O error if the file cannot be read or written.
    /// Returns a verification error if an existing file is not exactly 32 bytes.
    pub fn load_or_create(path: &std::path::Path) -> Result<[u8; 32]> {
        match std::fs::read(path) {
            Ok(bytes) => Self::seed_from_bytes(&bytes),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                let seed = Self::generate_seed();
                Self::write_seed(path, &seed)?;
                Ok(seed)
            }
            Err(e) => Err(SchubertError::Io(e)),
        }
    }

    /// Load an existing key seed. Fails if the file is absent or malformed.
    ///
    /// # Errors
    ///
    /// Returns an I/O error if the file cannot be read.
    /// Returns a verification error if the file is not exactly 32 bytes.
    pub fn load(path: &std::path::Path) -> Result<[u8; 32]> {
        let bytes = std::fs::read(path)?;
        Self::seed_from_bytes(&bytes)
    }

    /// Generate a fresh random 32-byte seed using the OS CSPRNG.
    pub fn generate_seed() -> [u8; 32] {
        use rand::RngCore;
        let mut seed = [0u8; 32];
        OsRng.fill_bytes(&mut seed);
        seed
    }

    fn seed_from_bytes(bytes: &[u8]) -> Result<[u8; 32]> {
        bytes.try_into().map_err(|_| {
            SchubertError::CryptoVerificationFailed("key file must be exactly 32 bytes".into())
        })
    }

    fn write_seed(path: &std::path::Path, seed: &[u8; 32]) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        Self::write_secret_file(path, seed)?;
        #[cfg(unix)]
        Self::set_owner_only(path)?;
        Ok(())
    }

    #[cfg(unix)]
    fn set_owner_only(path: &std::path::Path) -> Result<()> {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))?;
        Ok(())
    }

    #[cfg(unix)]
    fn write_secret_file(path: &std::path::Path, seed: &[u8; 32]) -> Result<()> {
        use std::io::Write;
        use std::os::unix::fs::OpenOptionsExt;
        let mut f = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(path)?;
        f.write_all(seed)?;
        Ok(())
    }

    #[cfg(not(unix))]
    fn write_secret_file(path: &std::path::Path, seed: &[u8; 32]) -> Result<()> {
        std::fs::write(path, seed)?;
        Ok(())
    }
}

// ---------- wire format helpers ----------

fn read_u16(buf: &[u8], pos: &mut usize) -> Result<usize> {
    if *pos + 2 > buf.len() {
        return Err(SchubertError::CryptoVerificationFailed(
            "truncated token: expected u16".into(),
        ));
    }
    let v = u16::from_be_bytes([buf[*pos], buf[*pos + 1]]) as usize;
    *pos += 2;
    Ok(v)
}

fn read_u8(buf: &[u8], pos: &mut usize) -> Result<usize> {
    if *pos >= buf.len() {
        return Err(SchubertError::CryptoVerificationFailed(
            "truncated token: expected u8".into(),
        ));
    }
    let v = buf[*pos] as usize;
    *pos += 1;
    Ok(v)
}

fn read_str(buf: &[u8], pos: &mut usize, len: usize) -> Result<String> {
    let bytes = read_bytes(buf, pos, len)?;
    String::from_utf8(bytes.to_vec())
        .map_err(|e| SchubertError::CryptoVerificationFailed(format!("non-utf8 token field: {e}")))
}

fn read_bytes<'a>(buf: &'a [u8], pos: &mut usize, len: usize) -> Result<&'a [u8]> {
    if *pos + len > buf.len() {
        return Err(SchubertError::CryptoVerificationFailed(
            "truncated token field".into(),
        ));
    }
    let slice = &buf[*pos..*pos + len];
    *pos += len;
    Ok(slice)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn issue_and_verify() {
        let issuer = CapabilityIssuer::generate();
        let token = issuer.issue("alice", "read:data").unwrap();

        let verifier = CapabilityVerifier::new(issuer.public_key());
        verifier.verify(&token).unwrap();
    }

    #[test]
    fn verify_wrong_key_fails() {
        let issuer1 = CapabilityIssuer::generate();
        let issuer2 = CapabilityIssuer::generate();
        let token = issuer1.issue("alice", "read:data").unwrap();

        let wrong_verifier = CapabilityVerifier::new(issuer2.public_key());
        assert!(wrong_verifier.verify(&token).is_err());
    }

    #[test]
    fn tampered_token_fails() {
        let issuer = CapabilityIssuer::generate();
        let mut token = issuer.issue("alice", "read:data").unwrap();

        // Tamper with capability
        token.capability = CapabilityId::new("write:data");

        let verifier = CapabilityVerifier::new(issuer.public_key());
        assert!(verifier.verify(&token).is_err());
    }

    #[test]
    fn tampered_principal_fails() {
        let issuer = CapabilityIssuer::generate();
        let mut token = issuer.issue("alice", "read:data").unwrap();

        token.principal = PrincipalId::new("bob");

        let verifier = CapabilityVerifier::new(issuer.public_key());
        assert!(verifier.verify(&token).is_err());
    }

    #[test]
    fn verify_and_extract() {
        let issuer = CapabilityIssuer::generate();
        let token = issuer.issue("alice", "read:data").unwrap();

        let verifier = CapabilityVerifier::new(issuer.public_key());
        let (pid, cid) = verifier.verify_and_extract(&token).unwrap();
        assert_eq!(pid.as_str(), "alice");
        assert_eq!(cid.as_str(), "read:data");
    }

    #[test]
    fn issue_batch() {
        let issuer = CapabilityIssuer::generate();
        let tokens = issuer
            .issue_batch(&[("alice", "read:data"), ("bob", "write:data")])
            .unwrap();

        assert_eq!(tokens.len(), 2);

        let verifier = CapabilityVerifier::new(issuer.public_key());
        for token in &tokens {
            verifier.verify(token).unwrap();
        }
    }

    // --- #16.1: to_bytes / from_bytes ---

    #[test]
    fn token_to_bytes_roundtrips() {
        let issuer = CapabilityIssuer::generate();
        let token = issuer.issue("alice", "read:data").unwrap();

        let bytes = CapabilityToken::to_bytes(&token);
        let decoded = CapabilityToken::from_bytes(&bytes).unwrap();

        assert_eq!(decoded.principal.as_str(), "alice");
        assert_eq!(decoded.capability.as_str(), "read:data");
        assert_eq!(decoded.issuer_key, token.issuer_key);
        assert_eq!(decoded.signature, token.signature);
    }

    #[test]
    fn token_from_bytes_rejects_empty() {
        assert!(CapabilityToken::from_bytes(&[]).is_err());
    }

    #[test]
    fn token_from_bytes_rejects_truncated() {
        let issuer = CapabilityIssuer::generate();
        let token = issuer.issue("alice", "read:data").unwrap();
        let bytes = CapabilityToken::to_bytes(&token);
        // Truncate last byte
        assert!(CapabilityToken::from_bytes(&bytes[..bytes.len() - 1]).is_err());
    }

    // --- #16.2: from_seed + public_key_hex ---

    #[test]
    fn issuer_from_seed_is_deterministic() {
        let seed = [42u8; 32];
        let issuer1 = CapabilityIssuer::from_seed(seed);
        let issuer2 = CapabilityIssuer::from_seed(seed);
        assert_eq!(issuer1.public_key(), issuer2.public_key());
    }

    #[test]
    fn issuer_from_seed_produces_usable_keys() {
        let seed = [7u8; 32];
        let issuer = CapabilityIssuer::from_seed(seed);
        let token = issuer.issue("alice", "read:data").unwrap();

        let verifier = CapabilityVerifier::new(issuer.public_key());
        verifier.verify(&token).unwrap();
    }

    #[test]
    fn public_key_hex_is_64_lowercase_chars() {
        let seed = [1u8; 32];
        let issuer = CapabilityIssuer::from_seed(seed);
        let hex = issuer.public_key_hex();
        assert_eq!(hex.len(), 64);
        assert!(hex
            .chars()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()));
    }

    #[test]
    fn public_key_hex_matches_raw() {
        let issuer = CapabilityIssuer::generate();
        let hex = issuer.public_key_hex();
        let raw = issuer.public_key();
        let expected: String = raw.iter().map(|b| format!("{b:02x}")).collect();
        assert_eq!(hex, expected);
    }

    // --- #16.4: Multi-capability grant tokens ---

    fn grant_cap(id: &str, partition: Vec<usize>) -> (CapabilityId, Vec<usize>) {
        (CapabilityId::new(id), partition)
    }

    #[test]
    fn issue_grant_and_verify() {
        let issuer = CapabilityIssuer::generate();
        let grant = issuer
            .issue_grant(
                "alice",
                &[
                    grant_cap("memory:read", vec![1]),
                    grant_cap("memory:write", vec![2]),
                ],
            )
            .unwrap();

        let verifier = GrantVerifier::new(issuer.public_key());
        verifier.verify(&grant).unwrap();
    }

    #[test]
    fn grant_may_single_cap() {
        let issuer = CapabilityIssuer::generate();
        let grant = issuer
            .issue_grant("alice", &[grant_cap("memory:read", vec![1])])
            .unwrap();

        let verifier = GrantVerifier::new(issuer.public_key());
        assert!(verifier.may(&grant, &[1]));
        assert!(!verifier.may(&grant, &[2]));
    }

    #[test]
    fn grant_may_write_implies_read() {
        // Geometric containment: σ₂ ≥ σ₁ component-wise, so write implies read.
        let issuer = CapabilityIssuer::generate();
        let grant = issuer
            .issue_grant("alice", &[grant_cap("memory:write", vec![2])])
            .unwrap();

        let verifier = GrantVerifier::new(issuer.public_key());
        assert!(verifier.may(&grant, &[1])); // read implied by write
        assert!(verifier.may(&grant, &[2])); // write explicitly granted
        assert!(!verifier.may(&grant, &[2, 1])); // manage not granted
    }

    #[test]
    fn grant_may_admin_implies_all() {
        // σ₄₄₄₄ is the maximum partition — everything ≤ it component-wise.
        let issuer = CapabilityIssuer::generate();
        let grant = issuer
            .issue_grant("alice", &[grant_cap("admin", vec![4, 4, 4, 4])])
            .unwrap();

        let verifier = GrantVerifier::new(issuer.public_key());
        assert!(verifier.may(&grant, &[1]));
        assert!(verifier.may(&grant, &[2]));
        assert!(verifier.may(&grant, &[3, 1]));
        assert!(verifier.may(&grant, &[4, 4, 4, 4]));
    }

    #[test]
    fn grant_roundtrip_bytes() {
        let issuer = CapabilityIssuer::generate();
        let grant = issuer
            .issue_grant(
                "alice",
                &[
                    grant_cap("memory:read", vec![1]),
                    grant_cap("memory:write", vec![2]),
                ],
            )
            .unwrap();

        let bytes = GrantToken::to_bytes(&grant);
        let decoded = GrantToken::from_bytes(&bytes).unwrap();

        assert_eq!(decoded.principal.as_str(), "alice");
        assert_eq!(decoded.capabilities.len(), 2);
    }

    #[test]
    fn grant_tampered_caps_fails_verify() {
        let issuer = CapabilityIssuer::generate();
        let mut grant = issuer
            .issue_grant("alice", &[grant_cap("memory:read", vec![1])])
            .unwrap();

        grant.capabilities.push(GrantCapability {
            id: CapabilityId::new("memory:write"),
            partition: vec![2],
        });

        let verifier = GrantVerifier::new(issuer.public_key());
        assert!(verifier.verify(&grant).is_err());
    }

    #[test]
    fn grant_singleton_is_backward_compatible() {
        let seed = [99u8; 32];
        let issuer = CapabilityIssuer::from_seed(seed);

        let single = issuer.issue("alice", "read:data").unwrap();
        let grant = issuer
            .issue_grant("alice", &[grant_cap("read:data", vec![1])])
            .unwrap();

        // Both verify cryptographically
        let verifier = CapabilityVerifier::new(issuer.public_key());
        verifier.verify(&single).unwrap();
        let gv = GrantVerifier::new(issuer.public_key());
        gv.verify(&grant).unwrap();

        // Grant may() with original partition should pass
        assert!(gv.may(&grant, &[1]));
    }

    #[test]
    fn grant_issue_batch_still_works() {
        let issuer = CapabilityIssuer::generate();
        let tokens = issuer
            .issue_batch(&[("alice", "read:data"), ("bob", "write:data")])
            .unwrap();
        assert_eq!(tokens.len(), 2);
    }

    // --- #16.5: KeyStore ---

    #[test]
    fn keystore_load_or_create_then_load_roundtrip() {
        let dir = tempfile_dir();
        let path = dir.join("issuer.key");
        let created = KeyStore::load_or_create(&path).unwrap();
        // Second call must read the same seed, not regenerate.
        let loaded = KeyStore::load_or_create(&path).unwrap();
        assert_eq!(created, loaded);
        let direct = KeyStore::load(&path).unwrap();
        assert_eq!(created, direct);
    }

    #[test]
    fn keystore_load_missing_file_fails() {
        let dir = tempfile_dir();
        let path = dir.join("nonexistent.key");
        assert!(KeyStore::load(&path).is_err());
    }

    #[test]
    fn keystore_wrong_size_file_rejected() {
        let dir = tempfile_dir();
        let path = dir.join("short.key");
        std::fs::write(&path, b"too short").unwrap();
        assert!(KeyStore::load(&path).is_err());
    }

    #[test]
    fn keystore_generate_seed_is_random() {
        let s1 = KeyStore::generate_seed();
        let s2 = KeyStore::generate_seed();
        assert_ne!(s1, s2);
    }

    fn tempfile_dir() -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "schubert-keystore-test-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }
}
