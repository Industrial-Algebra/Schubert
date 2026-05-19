# Golden Verification Files

This directory holds proof artifacts produced by Karpal verification backends.

## Files

| File | Source | Description |
|---|---|---|
| `schubert_lr_gr24.smt2` | SMT-LIB2 export | LR coefficients for Gr(2,4) |
| `schubert_lr_gr36.smt2` | SMT-LIB2 export | LR coefficients for Gr(3,6) |
| `SchubertVerify.lean` | Lean 4 export | Schubert calculus theorems |
| `schubert_certificates.json` | Certificate registry | Trust boundary certificates |

## Regeneration

When karpal-verify ships:
```bash
cargo test -p schubert --features karpal-verify -- --ignored --test export_golden
```

## Status

These files will be populated when karpal-verify Phase 12 completes.
Currently a forward-looking directory.
