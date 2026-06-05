# Verification Integration (Karpal)

Schubert integrates with Karpal (v0.5) for formal verification of access control
properties. Enable the `karpal-verify` feature.

## Architecture

```
AccessControl ──► verify.rs ──► karpal-verify (SMT/Lean)
                      │
                      ├── ObligationBundle ──► Proof obligations
                      ├── Certified<T>    ──► Trust boundary
                      └── VerificationResult ──► Pass/Fail/Caveat
```

## Obligation Bundles

Five obligation bundles verify key properties:

| Bundle | Property |
|---|---|
| `grant_check_consistency` | If granted, check must return Granted |
| `revoke_removes_access` | Revoke → check returns Denied or Impossible |
| `grant_revoke_identity` | Grant then revoke = no net change |
| `composition_associativity` | (a ∘ b) ∘ c = a ∘ (b ∘ c) |
| `impossibility_symmetry` | a impossible with b ⇔ b impossible with a |

## Certified Trust Boundary

```rust
use schubert::verify::Certified;

// Wrap a value in a proof obligation
let certified_decision: Certified<AccessDecision> =
    verify.check_certified(&acl, &principal, &["read"])?;
```

`Certified<T>` carries a formal proof that T satisfies specified properties.
At the boundary, the proof is discharged or rejected.

## Verification Levels

| Level | Backend | Guarantee |
|---|---|---|
| `QuickCheck` | Property testing | Statistical confidence |
| `SMT` | Z3/CVC4 | Symbolic model checking |
| `Lean` | Lean 4 | Full formal proof |

## Integration

```rust
use schubert::verify;
use karpal_verify::Verifier;

let verifier = Verifier::new();
let obligations = verify::build_obligations(&acl)?;

for obligation in &obligations {
    let result = verifier.verify(obligation)?;
    match result {
        verify::VerificationResult::Pass => {},
        verify::VerificationResult::Fail(reason) => {
            eprintln!("Verification failed: {reason}");
        },
        verify::VerificationResult::Caveat(msg) => {
            println!("Caveat: {msg}");
        },
    }
}
```
