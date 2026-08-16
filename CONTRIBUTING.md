# Contributing to Schubert

Thank you for your interest in contributing! Schubert is an Industrial Algebra
project, licensed under Apache-2.0.

## Contributor License Agreement (CLA)

**All contributors must sign a Contributor License Agreement (CLA).**

The CLA grants Industrial Algebra the right to relicense your contributions
under any license, while you retain full copyright ownership. This enables
Industrial Algebra to offer commercial licensing for the combined work.

### How to Sign

1. Download the CLA from: https://industrial-algebra.org/cla
2. Sign and email to: cla@industrial-algebra.org
3. Include your GitHub username in the email

Pull requests from contributors who have not signed the CLA cannot be merged.

## Development Setup

```bash
# Clone and build
git clone https://github.com/industrial-algebra/Schubert
cd Schubert
cargo build
cargo test
cargo clippy --all-targets
```

## Conventions

- **Rust edition 2021**, nightly toolchain
- `#![warn(missing_docs)]` — every public item must be documented
- `#![warn(clippy::all)]` — zero clippy warnings
- Tests use Gr(2,4) — the standard Grassmannian
- Feature gates are additive — never break existing API

## Pull Request Process

1. Sign the CLA (see above)
2. Ensure `cargo test --all-features` passes
3. Ensure `cargo clippy --all-targets --all-features` is clean
4. Add tests for new functionality
5. Update documentation (module docs, README if applicable)

## License

By contributing, you agree that your contributions will be licensed under
Apache-2.0, the same license as the project.
