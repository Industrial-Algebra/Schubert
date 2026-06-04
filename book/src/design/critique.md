# Critique & Future Work

> **v0.1.0 Snapshot** — This is an honest assessment of the project at its initial
> public release. Several points have been addressed since the original critique;
> we track the remainder as future work.

## Addressed Since Original Critique

| Concern | Resolution |
|---|---|
| No CLI tooling | ✅ CLI with `discover`, `recommend`, `explore` subcommands |
| AGPL-only licensing | ✅ Dual-licensed AGPL-3.0 + commercial |
| Sparse documentation | ✅ User guide, book, API reference, cookbook |
| Feature-flag complexity | ✅ Feature flag guide with common combinations |

## Ongoing Challenges

### Learning Curve
Schubert calculus is not standard security engineering knowledge. We recommend:
- Start with Gr(2,4) — the standard RBAC space
- Use the [Getting Started](../getting-started.md) walkthrough
- Use `schubert discover` to explore the API surface
- Read [Mathematical Foundation](../concepts/math.md) for the geometry

### Performance Benchmarks
No published benchmarks yet. We expect:
- Single check: <1ms for Gr(2,4)
- Batch check (100 principals): <10ms with `parallel` feature
- Tropical path: linear scaling in number of conditions

Benchmarks are tracked for a future release.

### Persistence
No built-in storage layer. All state is in-memory. Use `serde` serialization +
your database of choice for persistence.

### Real-World Adoption
Schubert is a new project. We welcome:
- Production deployment reports
- Bug reports and edge cases
- Integration examples with common stacks (PostgreSQL, Redis, Kubernetes)

## Future Directions

See the full [Roadmap](./roadmap.md) for speculative directions including:
- Persistent backends (SQLite, PostgreSQL)
- gRPC policy distribution protocol
- Policy diff and incremental updates
- Visualization of Schubert varieties
- Integration with OpenFGA / Rego policy languages
