# Policy Loader

Loading an access controller from a TOML policy file.

**Source**: `examples/policy_loader.rs`

## Pattern

```rust
let toml_str = std::fs::read_to_string("policy.toml")?;
let acl = AccessController::from_policy_toml(&toml_str)?;

// Use the loaded controller
let alice = acl.get_principal("alice")?;
let result = acl.check(&alice, &["read"])?;

// Export current state
let exported = acl.to_policy_toml()?;
std::fs::write("exported.toml", exported)?;
```

## Key Takeaway

Policies-as-code enable version control, code review, and CI validation of
access control configurations.
