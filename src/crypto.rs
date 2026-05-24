// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: AGPL-3.0-only

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
use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use rand::rngs::OsRng;

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

    /// Return the public key for distribution to verifiers.
    pub fn public_key(&self) -> Vec<u8> {
        self.signing_key.verifying_key().to_bytes().to_vec()
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
}
