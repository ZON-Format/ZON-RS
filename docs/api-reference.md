# ZON API Reference

Copyright (c) 2025 ZON-FORMAT (Roni Bhakta)

Complete API documentation for `zon-format` v1.0.5 (Rust).

## Installation

```toml
[dependencies]
zon-format = "1.0.5"
```

---

## Encoding Functions

### `encode(data: &Value) -> ZonEncodeResult<String>`

Encodes a serde_json Value to ZON format.

**Parameters:**
- `data` (`&serde_json::Value`) - JSON data to encode

**Returns:** `Result<String, ZonEncodeError>` - ZON-formatted string

**Example:**
```rust
use zon_format::encode;
use serde_json::json;

let data = json!({
  "users": [
    { "id": 1, "name": "Alice", "active": true },
    { "id": 2, "name": "Bob", "active": false }
  ]
});

let encoded = encode(&data).unwrap();
println!("{}", encoded);
```

**Output:**
```zon
users:@(2):active,id,name
T,1,Alice
F,2,Bob
```

**Supported Types:**
- ✅ Objects (nested or flat)
- ✅ Arrays (uniform, mixed, primitives)
- ✅ Strings
- ✅ Numbers (integers, floats)
- ✅ Booleans (`T`/`F`)
- ✅ Null (`null`)

---

## Decoding Functions

### `decode(zon_string: &str) -> ZonResult<Value>`

Decodes a ZON format string back to serde_json Value.

### Parameters

- **`zon_string`** (`&str`): The ZON-formatted string to decode

### Returns

`Result<serde_json::Value, ZonDecodeError>`

### Example

```rust
use zon_format::decode;

let zon_data = "name:Alice\nage:30\nactive:T";
let decoded = decode(zon_data).unwrap();

assert_eq!(decoded["name"], "Alice");
assert_eq!(decoded["age"], 30);
assert_eq!(decoded["active"], true);
```

---

### `decode_with_options(zon_string: &str, options: DecodeOptions) -> ZonResult<Value>`

Decodes with custom options.

### DecodeOptions

```rust
use zon_format::{decode_with_options, DecodeOptions};

// Strict mode (default) - throws on validation errors
let result = decode_with_options(zon_data, DecodeOptions::new().strict(true));

// Non-strict mode - allows mismatches
let result = decode_with_options(zon_data, DecodeOptions::new().strict(false));
```

---

## Schema Validation API

ZON provides a runtime schema validation library for LLM guardrails.

### Schema Builders

```rust
use zon_format::schema::{zon, ZonSchema};

// String schema
let s = zon::string();

// Number schema
let n = zon::number();

// Boolean schema
let b = zon::boolean();

// Enum schema
let e = zon::enum_values(vec!["admin", "user"]);

// Array schema
let arr = zon::array(zon::number());

// Object schema
let obj = zon::object(vec![
    ("name", Box::new(zon::string()) as Box<dyn ZonSchema>),
    ("age", Box::new(zon::number()) as Box<dyn ZonSchema>),
]);
```

### Modifiers

```rust
// Optional field
let optional_name = zon::string().optional();

// With description (for prompt generation)
let name = zon::string().describe("Full name of the user");
```

### Validation

```rust
use zon_format::schema::{validate, validate_value, ZonResult};

// Validate from ZON string
let result = validate("name:Alice\nage:30", &schema);

// Validate from Value directly
let result = validate_value(&json!({"name": "Alice", "age": 30}), &schema);

match result {
    ZonResult::Success { data } => println!("Valid: {:?}", data),
    ZonResult::Failure { error, issues } => println!("Error: {}", error),
}
```

### Prompt Generation

```rust
let prompt = schema.to_prompt(0);
// object:
//   - name: string - Full name of the user
//   - age: number
```

---

## Error Types

### `ZonDecodeError`

```rust
pub struct ZonDecodeError {
    pub message: String,
    pub details: ZonDecodeErrorDetails,
}

impl ZonDecodeError {
    pub fn code(&self) -> Option<&str>;
    pub fn line(&self) -> Option<usize>;
    pub fn column(&self) -> Option<usize>;
    pub fn context(&self) -> Option<&str>;
}
```

### `ZonEncodeError`

```rust
pub enum ZonEncodeError {
    CircularReference,
    UnsupportedType(String),
    EncodingError(String),
}
```

---

## Error Codes

| Code | Description | Example |
|------|-------------|----------|
| `E001` | Row count mismatch | Declared `@(3)` but only 2 rows provided |
| `E002` | Field count mismatch | Declared 3 columns but row has 2 values |
| `E301` | Document size exceeds 100MB | Prevents memory exhaustion |
| `E302` | Line length exceeds 1MB | Prevents buffer overflow |
| `E303` | Array length exceeds 1M items | Prevents excessive iteration |
| `E304` | Object key count exceeds 100K | Prevents hash collision |

---

## Constants

```rust
use zon_format::{
    TABLE_MARKER,      // '@'
    META_SEPARATOR,    // ':'
    MAX_DOCUMENT_SIZE, // 100 MB
    MAX_LINE_LENGTH,   // 1 MB
    MAX_ARRAY_LENGTH,  // 1,000,000
    MAX_OBJECT_KEYS,   // 100,000
    MAX_NESTING_DEPTH, // 100
};
```

---

## Complete Example

```rust
use zon_format::{encode, decode};
use serde_json::json;

fn main() {
    // Complex data structure
    let data = json!({
        "metadata": { "version": "1.0.5", "env": "production" },
        "users": [
            { "id": 1, "name": "Alice", "active": true, "loginCount": 42 },
            { "id": 2, "name": "Bob", "active": true, "loginCount": 17 },
            { "id": 3, "name": "Carol", "active": false, "loginCount": 3 }
        ],
        "config": { "database": { "host": "localhost", "port": 5432 } }
    });

    // Encode
    let encoded = encode(&data).unwrap();
    println!("Encoded:\n{}\n", encoded);

    // Decode
    let decoded = decode(&encoded).unwrap();
    
    // Verify round-trip
    assert_eq!(data, decoded);
    println!("Round-trip successful!");
}
```

---

## See Also

- [Syntax Cheatsheet](./syntax-cheatsheet.md) - Quick reference
- [Format Specification](../SPEC.md) - Formal grammar
- [LLM Best Practices](./llm-best-practices.md) - Usage guide
- [GitHub Repository](https://github.com/ZON-Format/ZON-RS)
- [Crates.io Package](https://crates.io/crates/zon-format)
