# WebAssembly

JavaScript bindings for in-browser access control. Enable the `wasm` feature.

## Build

```bash
cargo build --target wasm32-unknown-unknown --features wasm
```

## WasmController

```javascript
import init, { WasmController } from 'schubert';

await init();

const acl = new WasmController(2, 4);
acl.register_capability("read", [1], "ReadLike", "Read data");
acl.register_capability("write", [2], "WriteLike", "Write data");

const alice = acl.create_principal("alice");
acl.grant(alice, "read");
acl.grant(alice, "write");

const decision = acl.check(alice, ["read", "write"]);
// { kind: "Granted", configurations: 1 }
```

> **Note**: `AuditSink` is not available on wasm32 since it requires `std`.

## CapabilityKind Values

| JS String | Variant |
|---|---|
| `"ReadLike"` | Read-like capability |
| `"WriteLike"` | Write-like capability |
| `"AdminLike"` | Admin-like capability |
| `"Custom"` | Custom capability |

## Limitations

- Single-threaded (browser context)
- No audit sink (requires std)
- No parallel batch operations (requires rayon/threads)
