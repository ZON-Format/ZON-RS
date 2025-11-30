# ZON Specification

## Zero Overhead Notation - Formal Specification

**Version:** 1.0.5

**Date:** 2025-11-30

**Status:** Stable Release

**Authors:** ZON Format Contributors

**License:** MIT

---

## Abstract

Zero Overhead Notation (ZON) is a compact, line-oriented text format that encodes the JSON data model with minimal redundancy optimized for large language model token efficiency. ZON achieves up to 23.8% token reduction compared to JSON through single-character primitives (`T`, `F`), null as `null`, explicit table markers (`@`), colon-less nested structures, and intelligent quoting rules. Arrays of uniform objects use tabular encoding with column headers declared once; metadata uses flat key-value pairs. This specification defines ZON's concrete syntax, canonical value formatting, encoding/decoding behavior, conformance requirements, and strict validation rules. ZON provides deterministic, lossless representation achieving 100% LLM retrieval accuracy in benchmarks.

## Status of This Document

This document is a **Stable Release v1.0.5** and defines normative behavior for ZON encoders, decoders, and validators. Implementation feedback should be reported at https://github.com/ZON-Format/ZON-RS.

Backward compatibility is maintained across v1.0.x releases. Major versions (v2.x) may introduce breaking changes.

## Normative References

**[RFC2119]** Bradner, S., "Key words for use in RFCs to Indicate Requirement Levels", BCP 14, RFC 2119, March 1997.
https://www.rfc-editor.org/rfc/rfc2119

**[RFC8174]** Leiba, B., "Ambiguity of Uppercase vs Lowercase in RFC 2119 Key Words", BCP 14, RFC 8174, May 2017.
https://www.rfc-editor.org/rfc/rfc8174

**[RFC8259]** Bray, T., "The JavaScript Object Notation (JSON) Data Interchange Format", STD 90, RFC 8259, December 2017.
https://www.rfc-editor.org/rfc/rfc8259

## Informative References

**[RFC4180]** Shafranovich, Y., "Common Format and MIME Type for Comma-Separated Values (CSV) Files", RFC 4180, October 2005.
https://www.rfc-editor.org/rfc/rfc4180

**[ISO8601]** ISO 8601:2019, "Date and time — Representations for information interchange".

**[UNICODE]** The Unicode Consortium, "The Unicode Standard", Version 15.1, September 2023.

---

## Table of Contents

1. [Introduction](#introduction)
2. [Terminology and Conventions](#1-terminology-and-conventions)
3. [Data Model](#2-data-model)
4. [Encoding Normalization](#3-encoding-normalization)
5. [Decoding Interpretation](#4-decoding-interpretation)
6. [Concrete Syntax](#5-concrete-syntax)
7. [Primitives](#6-primitives)
8. [Strings and Keys](#7-strings-and-keys)
9. [Objects](#8-objects)
10. [Arrays](#9-arrays)
11. [Table Format](#10-table-format)
12. [Quoting and Escaping](#11-quoting-and-escaping)
13. [Whitespace](#12-whitespace-and-line-endings)
14. [Conformance](#13-conformance-and-options)
15. [Strict Mode Errors](#14-strict-mode-errors)
16. [Security](#15-security-considerations)
17. [Internationalization](#16-internationalization)
18. [Interoperability](#17-interoperability)
19. [Media Type](#18-media-type)
20. [Appendices](#appendices)

---

## Introduction (Informative)

### Purpose

ZON addresses token bloat in JSON while maintaining structural fidelity. By declaring column headers once, using single-character tokens, and eliminating redundant punctuation, ZON achieves optimal compression for LLM contexts.

### Design Goals

1. **Minimize tokens** - Every character counts in LLM context windows
2. **Preserve structure** - 100% lossless round-trip conversion
3. **Human readable** - Debuggable, understandable format
4. **LLM friendly** - Explicit markers aid comprehension
5. **Deterministic** - Same input → same output
6. **Deep Nesting** - Efficiently handles complex, recursive structures

### Use Cases

✅ **Use ZON for:**
- LLM prompt contexts (RAG, few-shot examples)
- Log storage and analysis
- Configuration files
- Browser storage (localStorage)
- Tabular data interchange
- **Complex nested data structures** (ZON excels here)

❌ **Don't use ZON for:**
- Public REST APIs (use JSON for compatibility)
- Real-time streaming protocols (not yet supported)
- Files requiring comments (use YAML/JSONC)

### Example

**JSON (118 chars):**
```json
{"users":[{"id":1,"name":"Alice","active":true},{"id":2,"name":"Bob","active":false}]}
```

**ZON (64 chars, 46% reduction):**
```zon
users:@(2):active,id,name
T,1,Alice
F,2,Bob
```

---

## 1. Terminology and Conventions

### 1.1 RFC2119 Keywords

The keywords **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHALL NOT**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY**, and **OPTIONAL** are interpreted per [RFC2119] and [RFC8174].

### 1.2 Definitions

**ZON document** - UTF-8 text conforming to this specification

**Line** - Character sequence terminated by LF (`\n`)

**Key-value pair** - Line pattern: `key:value`

**Table** - Array of uniform objects with header + data rows

**Table header** - Pattern: `key:@(N):columns` or `@(N):columns`

**Meta separator** - Colon (`:`) separating keys/values

**Table marker** - At-sign (`@`) indicating table structure

**Primitive** - Boolean, null, number, or string (not object/array)

**Uniform array** - All elements are objects with identical keys

**Strict mode** - Validation enforcing row/column counts

---

## 2. Data Model

### 2.1 JSON Compatibility

ZON encodes the JSON data model:
- **Primitives**: `string | number | boolean | null`
- **Objects**: `{ [string]: JsonValue }`
- **Arrays**: `JsonValue[]`

### 2.2 Ordering

- **Arrays**: Order MUST be preserved exactly
- **Objects**: Key order MUST be preserved
  - Encoders SHOULD sort keys alphabetically
  - Decoders MUST preserve document order

### 2.3 Canonical Numbers

**Requirements for ENCODER:**

1. **No leading zeros:** `007` → invalid
2. **No trailing zeros:** `3.14000` → `3.14`
3. **No unnecessary decimals:** Integer `5` stays `5`, not `5.0`
4. **No scientific notation:** `1e6` → `1000000`, `1e-3` → `0.001`
5. **Special values map to null:**
   - `NaN` → `null`
   - `Infinity` → `null`
   - `-Infinity` → `null`

---

## 6. Primitives

### 6.1 Booleans

**Encoding:**
- `true` → `T`
- `false` → `F`

**Decoding:**
- `T` (case-sensitive) → `true`
- `F` (case-sensitive) → `false`

### 6.2 Null

**Encoding:**
- `null` → `null` (4-character literal)

**Decoding:**
- `null` → `null`
- Also accepts (case-insensitive): `none`, `nil`

### 6.3 Numbers

**Examples:**
```zon
age:30
price:19.99
score:-42
temp:98.6
large:1000000
```

---

## 10. Table Format

### 10.1 Header Syntax

**With key:**
```
users:@(2):active,id,name
```

**Root array:**
```
@(2):active,id,name
```

**Components:**
- `users` - Array key (optional for root)
- `@` - Table marker (REQUIRED)
- `(2)` - Row count (REQUIRED for strict mode)
- `:` - Separator (REQUIRED)
- `active,id,name` - Columns, comma-separated (REQUIRED)

### 10.2 Column Order

Columns SHOULD be sorted alphabetically.

### 10.3 Data Rows

Each row is comma-separated values:

```zon
T,1,Alice,admin
```

---

## 13. Conformance and Options

### 13.1 Encoder Checklist

✅ **A conforming encoder MUST:**

- [ ] Emit UTF-8 with LF line endings
- [ ] Encode booleans as `T`/`F`
- [ ] Encode null as `null`
- [ ] Emit canonical numbers (§2.3)
- [ ] Normalize NaN/Infinity to `null`
- [ ] Detect uniform arrays → table format
- [ ] Emit table headers: `key:@(N):columns`
- [ ] Sort columns alphabetically
- [ ] Quote strings per §7.2-7.3
- [ ] Preserve array order
- [ ] Ensure round-trip: `decode(encode(x)) === x`

### 13.2 Decoder Checklist

✅ **A conforming decoder MUST:**

- [ ] Accept UTF-8 (LF or CRLF)
- [ ] Decode `T` → true, `F` → false, `null` → null
- [ ] Parse decimal and exponent numbers
- [ ] Treat leading-zero numbers as strings
- [ ] Unescape quoted strings
- [ ] Parse table headers: `key:@(N):columns`
- [ ] Preserve array order
- [ ] Preserve key order
- [ ] **Error Codes:**
    - `E001`: Row count mismatch (strict mode)
    - `E002`: Field count mismatch (strict mode)
    - `E301`: Document size > 100MB
    - `E302`: Line length > 1MB
    - `E303`: Array length > 1M items
    - `E304`: Object key count > 100K

---

## 15. Security Considerations

### 15.1 Resource Limits

Implementations SHOULD limit:
- Document size: 100 MB
- Line length: 1 MB
- Nesting depth: 100 levels
- Array length: 1,000,000
- Object keys: 100,000

Prevents denial-of-service attacks.

---

## 18. Media Type & File Extension

### 18.1 File Extension

**Extension:** `.zonf`

### 18.2 Media Type

**Media type:** `text/zon`

**Charset:** UTF-8 (always)

---

## Appendix E: License

MIT License

Copyright (c) 2025 ZON-FORMAT (Roni Bhakta)

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

---

**End of Specification**

For the complete specification, please see the [zon-ts specification](https://github.com/ZON-Format/zon-ts/blob/main/SPEC.md).
