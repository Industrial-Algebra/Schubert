# Audit & Error

## AuditSink Trait

Pluggable audit interface for recording access decisions.

```rust
use schubert::audit::{AuditSink, DecisionRecord};

struct DatabaseAudit { pool: PgPool }

impl AuditSink for DatabaseAudit {
    fn record(&self, record: &DecisionRecord) -> schubert::Result<()> {
        // Write to database, file, log, etc.
        Ok(())
    }
}

acl.set_audit_sink(Box::new(DatabaseAudit { pool }));
```

## InMemoryAudit

Built-in audit sink that stores records in memory:

```rust
use schubert::audit::InMemoryAudit;

let sink = InMemoryAudit::new();
acl.set_audit_sink(Box::new(sink));

// After checks...
let records = sink.records();
let filtered = sink.records_for_principal(&alice);
sink.clear();
```

## Audit Design

- **Fire-and-forget**: Failing sinks never block access decisions
- **Feature-gated**: `AuditSink` requires the `std` feature
- **No_std**: `InMemoryAudit` uses `RefCell`, no thread safety

## SchubertError

All errors use `SchubertError` with 11 variants:

| Variant | When |
|---|---|
| `InvalidGrassmannian` | k ≥ n or k = 0 |
| `CapabilityNotFound` | Referenced capability not registered |
| `PrincipalNotFound` | Referenced principal doesn't exist |
| `AlreadyHolds` | Duplicate grant |
| `DoesNotHold` | Revoke on unheld capability |
| `InvalidPartition` | Partition not weakly decreasing |
| `ImpossibleComposition` | Geometric incompatibility |
| `Underconstrained` | Too few conditions |
| `Overconstrained` | Too many conditions |
| `SerializationError` | serde I/O failure |
| `VerificationError` | Karpal proof obligation failed |
