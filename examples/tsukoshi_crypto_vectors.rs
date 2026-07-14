// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! Emit cross-validation vectors for schubert-tsukoshi's TypeScript crypto.
//!
//! Issues single-capability and grant tokens with a **fixed seed**, then prints
//! them as JSON. The TypeScript test suite imports these vectors verbatim and
//! verifies them — proving byte-level Rust↔TS interop for the Ed25519 wire
//! format and signing message.
//!
//! Run:
//! ```text
//! cargo run --example tsukoshi_crypto_vectors
//! ```

use schubert::crypto::{CapabilityIssuer, CapabilityToken, GrantToken};
use schubert::{CapabilityId, PrincipalId};

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn main() {
    // Fixed, deterministic seed — shared with the TS test suite.
    let seed: [u8; 32] = [42u8; 32];
    let issuer = CapabilityIssuer::from_seed(seed);
    let pk_hex = issuer.public_key_hex();
    let pk_bytes = issuer.public_key();

    // --- Single-capability token ---
    let alice: PrincipalId = PrincipalId::new("alice");
    let cap_read = CapabilityId::new("memory:read");
    let token = issuer.issue(alice.clone(), cap_read.clone()).unwrap();
    let token_bytes = CapabilityToken::to_bytes(&token);

    // --- Multi-capability grant token ---
    // Deliberately passed in NON-canonical order to exercise the sort.
    let grant = issuer
        .issue_grant(
            "bob",
            &[
                (CapabilityId::new("memory:write"), vec![2]),
                (CapabilityId::new("memory:read"), vec![1]),
            ],
        )
        .unwrap();
    let grant_bytes = GrantToken::to_bytes(&grant);

    // --- Tampered grant (extra capability added post-sign) — must FAIL verify ---
    let mut tampered = issuer
        .issue_grant("carol", &[(CapabilityId::new("memory:read"), vec![1])])
        .unwrap();
    tampered.capabilities.push(schubert::crypto::GrantCapability {
        id: CapabilityId::new("memory:admin"),
        partition: vec![4, 4, 4, 4],
    });
    let tampered_bytes = GrantToken::to_bytes(&tampered);

    println!("{{");
    println!("  \"seed_hex\": \"{}\",", hex(&seed));
    println!("  \"public_key_hex\": \"{}\",", pk_hex);
    println!("  \"public_key_bytes\": {:?}", pk_bytes);
    println!("  \"single_principal\": \"alice\",");
    println!("  \"single_capability\": \"memory:read\",");
    println!("  \"single_token_hex\": \"{}\",", hex(&token_bytes));
    println!("  \"grant_principal\": \"bob\",");
    println!("  \"grant_capabilities\": [");
    println!("    {{\"id\":\"memory:read\",\"partition\":[1]}},");
    println!("    {{\"id\":\"memory:write\",\"partition\":[2]}}");
    println!("  ],");
    println!("  \"grant_token_hex\": \"{}\",", hex(&grant_bytes));
    println!("  \"tampered_token_hex\": \"{}\",", hex(&tampered_bytes));
    println!("  \"tampered_must_verify\": false");
    println!("}}");
}
