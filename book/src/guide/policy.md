# Policy Language (TOML)

Schubert supports declarative policies via TOML files. Enable with the `policy` feature.

## Policy File Format

```toml
# policy.toml
[grassmannian]
k = 2
n = 4

[capabilities.read]
partition = [1]
kind = "ReadLike"
label = "Read access"

[capabilities.write]
partition = [2]
kind = "WriteLike"
label = "Write access"

[capabilities.admin]
partition = [2, 2]
kind = "AdminLike"
label = "Full administration"

[principals.alice]
grants = ["read", "write"]

[principals.bob]
grants = ["read"]

[principals.admin_user]
grants = ["admin"]
```

## Loading Policies

```rust
use schubert::AccessController;

let toml_str = std::fs::read_to_string("policy.toml")?;
let acl = AccessController::from_policy_toml(&toml_str)?;

// Use the loaded controller
let alice = acl.get_principal("alice")?;
acl.check(&alice, &["read", "write"])?;
```

## Exporting Policies

```rust
let toml_str = acl.to_policy_toml()?;
std::fs::write("exported-policy.toml", toml_str)?;
```

## Validation

Policies are validated on load:

- Grassmannian dimensions must satisfy 0 < k < n
- Partitions must be weakly decreasing
- Capability IDs must be unique
- Principal grants must reference registered capabilities
- CapabilityKind must be a valid variant

Invalid policies return descriptive errors with context.
