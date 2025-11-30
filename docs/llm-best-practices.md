# Using ZON with LLMs - Best Practices

Copyright (c) 2025 ZON-FORMAT (Roni Bhakta)

Guide for maximizing ZON's effectiveness in LLM applications.

## Why ZON for LLMs?

LLM API costs are directly tied to token count. ZON reduces tokens by **23.8% vs JSON** while achieving **100% retrieval accuracy**.

**Key Benefits:**
- 💰 **Lower costs**: Fewer tokens = lower API bills
- 🎯 **Better accuracy**: 100% vs JSON's 91.7%
- 📊 **Self-documenting**: Explicit headers `@(N):columns`
- 🔍 **Human-readable**: Easy to debug and verify

---

## Sending ZON as Input

### Basic Pattern

Wrap ZON data in code blocks with format label:

````markdown
Here's the user data in ZON format:

```zon
users:@(3):active,id,name,role
T,1,Alice,admin
T,2,Bob,user
F,3,Carol,guest
```

Question: How many active users are there?
````

**Why this works:**
- ✅ Code blocks prevent formatting issues
- ✅ `zon` label helps model recognize format
- ✅ Explicit headers (`@(3):columns`) give clear schema

---

## Prompting Strategies

### Strategy 1: Show the Format (No Explanation)

**Best approach** - Let the model infer the structure:

````
```zon
products:@(4):category,id,name,price,stock
Electronics,1,Laptop,999,45
Books,2,Python Guide,29.99,120
Electronics,3,Mouse,19.99,200
Books,4,JavaScript Basics,24.95,85
```

Find products with stock below 100.
````

### Strategy 2: Minimal Context

For complex queries, add brief context:

````
Data format: ZON (tabular)
@(N) = row count
Column names listed in header

```zon
logs:@(100):level,message,timestamp,userId
ERROR,Database timeout,2025-01-15T10:30:00Z,1001
WARN,High memory usage,2025-01-15T10:31:15Z,1002
ERROR,API rate limit,2025-01-15T10:32:45Z,1001
...
```

How many ERROR logs are from userId 1001?
````

---

## Common Use Cases

### 1. Data Retrieval Questions

**Perfect for ZON** - table format excels here:

````
```zon
employees:@(20):active,department,id,name,salary
T,Engineering,1,Alice Chen,95000
T,Sales,2,Bob Smith,75000
F,Marketing,3,Carol Lee,68000
...
```

Questions:
1. What's the average salary in Engineering?
2. How many inactive employees are there?
3. List all Sales department employees.
````

### 2. Aggregation Tasks

````
```zon
transactions:@(1000):amount,category,date,userId
45.99,groceries,2025-01-10,1001
120.00,electronics,2025-01-10,1002
23.50,groceries,2025-01-11,1001
...
```

Calculate total spending by category for userId 1001.
````

### 3. Filtering and Search

````
```zon
products:@(500):category,inStock,name,price,rating
Electronics,T,Laptop Pro,1299,4.5
Books,F,Python Guide,29.99,4.8
Electronics,T,USB Mouse,19.99,4.2
...
```

Find all in-stock Electronics with rating above 4.0.
````

---

## Rust Integration Example

```rust
use zon_format::{encode, decode};
use serde_json::json;

// Prepare data for LLM context
let data = json!({
    "users": [
        {"id": 1, "name": "Alice", "active": true},
        {"id": 2, "name": "Bob", "active": false},
        {"id": 3, "name": "Carol", "active": true}
    ]
});

// Encode to ZON for LLM prompt
let zon_context = encode(&data).unwrap();

// Build prompt
let prompt = format!(
    "Here's the user data in ZON format:\n\n```zon\n{}\n```\n\nHow many active users are there?",
    zon_context
);

// Send to LLM...
// The LLM will see compact, self-documenting data
```

---

## Schema Validation for LLM Outputs

Use ZON's schema validation to ensure LLM outputs are correct:

```rust
use zon_format::schema::{zon, validate, ZonSchema};

// Define expected output schema
let schema = zon::object(vec![
    ("name", Box::new(zon::string()) as Box<dyn ZonSchema>),
    ("age", Box::new(zon::number()) as Box<dyn ZonSchema>),
    ("role", Box::new(zon::enum_values(vec!["admin", "user"])) as Box<dyn ZonSchema>),
]);

// Validate LLM output
let llm_output = "name:Alice\nage:30\nrole:admin";
let result = validate(llm_output, &schema);

if result.is_success() {
    println!("Valid output!");
} else {
    println!("Invalid output - retry with prompt adjustment");
}
```

---

## Benchmark Results

### Token Efficiency by Tokenizer

| Tokenizer | ZON Tokens | vs JSON | vs TOON | vs CSV |
|-----------|------------|---------|---------|--------|
| GPT-4o (o200k) | 143,661 | -23.8% | -36.1% | -12.9% |
| Claude 3.5 | 145,652 | -21.3% | -26.0% | -9.9% |
| Llama 3 | 230,838 | -16.5% | -26.7% | -9.2% |

### LLM Accuracy

```
ZON:           100% (306/309 questions)
TOON:          100% (306/309 questions)
CSV:           99%  (306/309 questions)
JSON compact:  91.7% (283/309 questions)
```

---

## Quick Reference

### Do's ✅
- Use code blocks for formatting
- Include `@(N)` row counts
- List column names explicitly
- Use `T`/`F` for booleans
- Use `null` for null values

### Don'ts ❌
- Don't explain ZON syntax (show, don't tell)
- Don't mix formats (stick to ZON)
- Don't omit row counts
- Don't use verbose field names unnecessarily

---

**See also:**
- [Syntax Cheatsheet](./syntax-cheatsheet.md) - Quick reference
- [API Reference](./api-reference.md) - encode/decode functions
- [Format Specification](../SPEC.md) - Formal grammar
