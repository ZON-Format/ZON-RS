# ZON-RS

**Zero Overhead Notation (ZON)** - A compact, human-readable data serialization format optimized for LLM token efficiency.

[![Crates.io](https://img.shields.io/crates/v/zon-format.svg)](https://crates.io/crates/zon-format)
[![Documentation](https://docs.rs/zon-format/badge.svg)](https://docs.rs/zon-format)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Overview

ZON is a line-oriented text format that encodes the JSON data model with minimal redundancy. It achieves **up to 35-50% token reduction** compared to JSON through:

- ✅ **Single-character primitives** (`T`, `F` for booleans)
- ✅ **Tabular array encoding** (headers declared once)
- ✅ **Intelligent quoting** (minimal quote overhead)
- ✅ **Explicit table markers** (`@(N):columns`)

### Key Features

| Feature | Description |
|---------|-------------|
| 🎯 **100% LLM Accuracy** | Self-explanatory format achieves perfect retrieval |
| 💾 **Most Token-Efficient** | 4-15% fewer tokens than TOON across all tokenizers |
| 🔒 **Security Limits** | Built-in DOS prevention (100MB docs, 1M arrays, 100K keys) |
| ✅ **Production Ready** | Comprehensive tests, zero data loss |

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
zon-format = "1.0.5"
```

## Quick Start

```rust
use zon_format::{encode, decode};
use serde_json::json;

fn main() {
    // Create some data
    let data = json!({
        "users": [
            {"id": 1, "name": "Alice", "active": true},
            {"id": 2, "name": "Bob", "active": false}
        ]
    });

    // Encode to ZON format
    let encoded = encode(&data).unwrap();
    println!("{}", encoded);
    // Output:
    // users:@(2):active,id,name
    // T,1,Alice
    // F,2,Bob

    // Decode back to JSON
    let decoded = decode(&encoded).unwrap();
    assert_eq!(data, decoded);
}
```

## Format Examples

### Basic Metadata

```zon
name:Alice
age:30
active:T
```

### Tables (Uniform Arrays)

```zon
users:@(3):active,id,name,role
T,1,Alice,admin
T,2,Bob,user
F,3,Carol,guest
```

**JSON equivalent:**
```json
{
  "users": [
    {"id": 1, "name": "Alice", "role": "admin", "active": true},
    {"id": 2, "name": "Bob", "role": "user", "active": true},
    {"id": 3, "name": "Carol", "role": "guest", "active": false}
  ]
}
```

### Nested Objects

```zon
config.database{host:localhost,port:5432}
config.cache{ttl:3600,enabled:T}
```

## API Reference

### Encoding

```rust
use zon_format::encode;
use serde_json::json;

let data = json!({"name": "Alice", "age": 30});
let encoded = encode(&data)?;
```

### Decoding

```rust
use zon_format::{decode, decode_with_options, DecodeOptions};

// Default (strict mode enabled)
let decoded = decode("name:Alice\nage:30")?;

// With custom options
let decoded = decode_with_options(
    "users:@(2):id,name\n1,Alice\n2,Bob",
    DecodeOptions::new().strict(false)
)?;
```

### Schema Validation

```rust
use zon_format::schema::{zon, validate, ZonSchema};

// Define a schema
let schema = zon::object(vec![
    ("name", Box::new(zon::string().describe("Full name"))),
    ("age", Box::new(zon::number().describe("Age in years"))),
    ("role", Box::new(zon::enum_values(vec!["admin", "user"]))),
]);

// Generate prompt for LLMs
let prompt = schema.to_prompt(0);
// object:
//   - name: string - Full name
//   - age: number - Age in years
//   - role: enum(admin, user)

// Validate input
let result = validate("name:Alice\nage:30\nrole:admin", &schema);
assert!(result.is_success());
```

## CLI Usage

```bash
# Build the CLI
cargo build --release

# Encode JSON to ZON
./target/release/zon encode data.json > data.zonf

# Decode ZON to JSON
./target/release/zon decode data.zonf > output.json
```

## Strict Mode

Strict mode (enabled by default) validates table structure:

| Code | Error | Description |
|------|-------|-------------|
| E001 | Row count mismatch | Declared vs actual row count differs |
| E002 | Field count mismatch | Row has fewer fields than columns |
| E301 | Document too large | Exceeds 100MB limit |
| E302 | Line too long | Exceeds 1MB limit |

```rust
// Disable strict mode to allow mismatches
let decoded = decode_with_options(data, DecodeOptions::new().strict(false))?;
```

## Benchmark Results

ZON achieves superior token efficiency across all major LLM tokenizers:

### GPT-4o (o200k)
```
ZON:          143,661 tokens 👑
  vs JSON:    -23.8%
  vs TOON:    -36.1%
  vs CSV:     -12.9%
  vs XML:     -57.1%
```

### Claude 3.5 (Anthropic)
```
ZON:          145,652 tokens 👑
  vs JSON:    -21.3%
  vs TOON:    -26.0%
  vs CSV:     -9.9%
  vs XML:     -55.5%
```

### Llama 3 (Meta)
```
ZON:          230,838 tokens 👑
  vs JSON:    -16.5%
  vs TOON:    -26.7%
  vs CSV:     -9.2%
  vs XML:     -51.9%
```

See [benchmarks/benchmark_output.md](./benchmarks/benchmark_output.md) for detailed results.

## Type Conversions

| ZON | JSON | Notes |
|-----|------|-------|
| `T` | `true` | Boolean true |
| `F` | `false` | Boolean false |
| `null` | `null` | Null value |
| `42` | `42` | Integer |
| `3.14` | `3.14` | Float |
| `hello` | `"hello"` | Unquoted string |
| `"hello"` | `"hello"` | Quoted string |

## Security

ZON includes built-in security limits to prevent denial-of-service attacks:

- **Document size**: 100 MB max
- **Line length**: 1 MB max
- **Array length**: 1,000,000 items max
- **Object keys**: 100,000 max
- **Nesting depth**: 100 levels max

## File Extension & Media Type

- **File Extension**: `.zonf`
- **Media Type**: `text/zon`
- **Encoding**: UTF-8 (always)

## License

MIT License - Copyright (c) 2025 ZON-FORMAT (Roni Bhakta)

See [LICENSE](./LICENSE) for details.

## Related Projects

- [zon-ts](https://github.com/ZON-Format/zon-ts) - TypeScript/JavaScript implementation

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## See Also

- [SPEC.md](./SPEC.md) - Formal specification
- [docs/api-reference.md](./docs/api-reference.md) - Full API documentation
- [docs/syntax-cheatsheet.md](./docs/syntax-cheatsheet.md) - Quick reference
- [docs/llm-best-practices.md](./docs/llm-best-practices.md) - LLM usage guide