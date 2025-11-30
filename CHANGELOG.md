# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.5] - 2025-11-30

### Added
- **Initial Rust Implementation**: Full port of ZON format from TypeScript
- **Encoder**: Complete ZON v2.0.0 encoder with:
  - Compact header syntax (`@(N):columns`)
  - Colon-less nested syntax (`key{...}` and `key[...]`)
  - Smart flattening with dot notation
  - Control character escaping
  - Sparse table encoding for semi-uniform data
- **Decoder**: Full ZON v2.0.0 decoder with:
  - Support for v1.x and v2.0.0 formats
  - Strict mode validation (E001, E002 error codes)
  - Security limits (E301-E304)
  - Case-insensitive boolean/null aliases
  - Dotted key unflattening
- **Schema Validation**: Runtime validation library for LLM guardrails
  - String, Number, Boolean, Enum types
  - Object and Array composite types
  - Optional field support
  - Prompt generation for LLMs
- **CLI Tool**: Command-line interface for encode/decode operations
- **Comprehensive Test Suite**: 100+ tests covering:
  - Round-trip codec tests
  - Encoder conformance tests
  - Decoder conformance tests
  - Strict mode validation
  - Canonical number formatting
  - Security limits
  - Schema validation
- **Documentation**: Full documentation including:
  - README with usage examples
  - SPEC.md formal specification
  - API reference
  - Syntax cheatsheet
  - LLM best practices guide
  - Benchmark results

### Features
- **100% LLM Accuracy**: Achieves perfect retrieval with self-explanatory structure
- **Token Efficiency**: 23.8% reduction vs JSON (GPT-4o), up to 36.1% vs TOON
- **Security**: Built-in DOS prevention limits
- **Lossless Round-Trip**: Zero data loss in encode/decode cycles

### Technical
- Built with Rust 2021 edition
- Dependencies: serde, serde_json (with preserve_order), thiserror, indexmap, regex
- MIT License

[1.0.5]: https://github.com/ZON-Format/ZON-RS/releases/tag/v1.0.5
