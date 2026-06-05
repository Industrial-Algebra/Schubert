# Security Considerations

## Identity Model

Schubert **never authenticates**. `PrincipalId` is an opaque string provided by your
external identity system (OAuth, OIDC, JWT, mTLS). Schubert authorizes based on
identities you provide.

**You are responsible for:**
- Authenticating users
- Mapping authenticated identities to `PrincipalId` values
- Ensuring identity consistency across service boundaries

## Capability Design

- **Partition collisions**: Two capabilities with the same partition are
  geometrically equivalent. Design partitions to be distinct.
- **Over-granting**: Granting too many capabilities may create impossible
  combinations. Use `analyze_stability()` to detect this.
- **Admin capabilities**: `[2,2]` (point class) is maximally restrictive. Grant
  admin capabilities sparingly.

## Trust Boundaries

- **`Certified<T>`**: A formal proof boundary. Values crossing this boundary have
  been verified by Karpal. Rejection means the proof obligation failed.
- **`AuditSink`**: Audit records are best-effort. A failing sink does not block
  access decisions.
- **`CapabilityToken`**: Ed25519 signatures provide integrity and authenticity but
  not confidentiality. Token contents are plaintext.

## Known Limitations

- **No built-in persistence**: Policy state is in-memory. Use `serde` +
  your own storage for durability.
- **No network protocol**: Schubert is purely a library. Implement wire protocols
  yourself.
- **Cosine similarity**: Holographic queries use approximate similarity,
  not exact access matching.
- **No revocation propagation**: CRDT revocations are eventually consistent,
  not immediately globally visible.

## Cryptographic Tokens

- **Key management**: `CapabilityIssuer` generates keys. Store and rotate keys
  according to your security policy.
- **No replay protection**: Tokens are stateless. Track used tokens in the verifier
  if replay is a concern.
- **Signature verification**: Always verify before extracting claims. Never trust
  unverified token contents.

## Performance Considerations

- **Grassmannian scaling**: Larger Grassmannians (Gr(3,6), Gr(4,8)) have higher
  computational cost for intersection calculations.
- **Batch operations**: Use `check_batch()` with the `parallel` feature for
  high-throughput scenarios.
- **Tropical path**: Switch to `ComputationPath::Tropical` for >1000 concurrent
  principals.
