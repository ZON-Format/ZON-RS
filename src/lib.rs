//! # ZON Format v1.0.5
//!
//! Zero Overhead Notation - A human-readable data serialization format
//! optimized for LLM token efficiency.
//!
//! ZON is a compact, line-oriented text format that encodes the JSON data model
//! with minimal redundancy. It achieves up to 35-50% token reduction compared to JSON
//! through single-character primitives (`T`, `F`), explicit table markers (`@`),
//! and intelligent compression while maintaining 100% data fidelity.
//!
//! ## Features
//!
//! - 🎯 **100% LLM Accuracy**: Achieves perfect retrieval with self-explanatory structure
//! - 💾 **Most Token-Efficient**: 4-15% fewer tokens than TOON across all tokenizers
//! - 🎯 **JSON Data Model**: Encodes the same objects, arrays, and primitives as JSON
//! - 📐 **Minimal Syntax**: Explicit headers eliminate ambiguity for LLMs
//! - 🧺 **Tabular Arrays**: Uniform arrays collapse into tables that declare fields once
//! - 🔢 **Canonical Numbers**: No scientific notation, NaN/Infinity → null
//! - 🔒 **Security Limits**: Automatic DOS prevention (100MB docs, 1M arrays, 100K keys)
//! - ✅ **Production Ready**: Comprehensive tests, zero data loss
//!
//! ## Quick Start
//!
//! ```rust
//! use zon_format::{encode, decode};
//! use serde_json::json;
//!
//! let data = json!({
//!     "users": [
//!         {"id": 1, "name": "Alice", "active": true},
//!         {"id": 2, "name": "Bob", "active": false}
//!     ]
//! });
//!
//! // Encode to ZON format
//! let encoded = encode(&data).unwrap();
//! println!("{}", encoded);
//! // users:@(2):active,id,name
//! // T,1,Alice
//! // F,2,Bob
//!
//! // Decode back to JSON
//! let decoded = decode(&encoded).unwrap();
//! assert_eq!(data, decoded);
//! ```
//!
//! ## Schema Validation
//!
//! ZON includes a runtime validation layer designed for LLM guardrails:
//!
//! ```rust
//! use zon_format::schema::{zon, validate, ZonSchema};
//!
//! let schema = zon::object(vec![
//!     ("name", Box::new(zon::string().describe("Full name"))),
//!     ("age", Box::new(zon::number().describe("Age in years"))),
//! ]);
//!
//! // Generate prompt for LLM
//! let prompt = schema.to_prompt(0);
//! // object:
//! //   - name: string - Full name
//! //   - age: number - Age in years
//!
//! // Validate LLM output
//! let result = validate("name:Alice\nage:30", &schema);
//! assert!(result.is_success());
//! ```
//!
//! ## File Extension and Media Type
//!
//! - **File Extension**: `.zonf`
//! - **Media Type**: `text/zon`
//! - **Encoding**: UTF-8
//!
//! ## License
//!
//! MIT License - Copyright (c) 2025 ZON-FORMAT (Roni Bhakta)

pub mod constants;
pub mod decoder;
pub mod encoder;
pub mod error;
pub mod schema;

// Re-export main types and functions
pub use constants::*;
pub use decoder::{decode, decode_with_options, DecodeOptions, ZonDecoder};
pub use encoder::{encode, encode_with_interval, ZonEncoder};
pub use error::{ZonDecodeError, ZonDecodeErrorDetails, ZonEncodeError, ZonEncodeResult, ZonResult};

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_roundtrip_empty_object() {
        let data = json!({});
        let encoded = encode(&data).unwrap();
        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_roundtrip_simple_metadata() {
        let data = json!({
            "name": "Alice",
            "age": 30,
            "active": true
        });
        let encoded = encode(&data).unwrap();
        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_roundtrip_nested_object() {
        let data = json!({
            "user": {
                "name": "Bob",
                "profile": {
                    "age": 25,
                    "city": "NYC"
                }
            }
        });
        let encoded = encode(&data).unwrap();
        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_roundtrip_array_of_objects() {
        let data = json!([
            {"id": 1, "name": "Alice", "score": 95},
            {"id": 2, "name": "Bob", "score": 87},
            {"id": 3, "name": "Charlie", "score": 92}
        ]);
        let encoded = encode(&data).unwrap();
        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_roundtrip_mixed_metadata_and_table() {
        let data = json!({
            "title": "Sales Report",
            "year": 2024,
            "records": [
                {"month": "Jan", "sales": 1000},
                {"month": "Feb", "sales": 1200},
                {"month": "Mar", "sales": 1100}
            ]
        });
        let encoded = encode(&data).unwrap();
        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_roundtrip_boolean_values() {
        let data = json!({
            "success": true,
            "error": false,
            "items": [
                {"id": 1, "active": true},
                {"id": 2, "active": false}
            ]
        });
        let encoded = encode(&data).unwrap();
        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_roundtrip_null_values() {
        let data = json!({
            "name": "Test",
            "value": null,
            "items": [
                {"id": 1, "data": null},
                {"id": 2, "data": "value"}
            ]
        });
        let encoded = encode(&data).unwrap();
        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_roundtrip_numbers() {
        let data = json!({
            "integer": 42,
            "float": 3.14,
            "negative": -10,
            "items": [
                {"id": 1, "value": 100},
                {"id": 2, "value": 200.5}
            ]
        });
        let encoded = encode(&data).unwrap();
        let decoded = decode(&encoded).unwrap();
        
        // Compare with tolerance for floats
        assert_eq!(decoded["integer"], data["integer"]);
        assert_eq!(decoded["negative"], data["negative"]);
        assert!((decoded["float"].as_f64().unwrap() - data["float"].as_f64().unwrap()).abs() < 0.001);
    }

    #[test]
    fn test_roundtrip_empty_arrays() {
        let data = json!({
            "empty": [],
            "nested": {
                "also_empty": []
            }
        });
        let encoded = encode(&data).unwrap();
        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_roundtrip_nested_arrays_in_metadata() {
        let data = json!({
            "tags": ["javascript", "typescript", "node"],
            "matrix": [[1, 2], [3, 4]],
            "items": [
                {"id": 1, "values": [10, 20]},
                {"id": 2, "values": [30, 40]}
            ]
        });
        let encoded = encode(&data).unwrap();
        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_roundtrip_complex_nested_objects_in_table() {
        let data = json!([
            {
                "id": 1,
                "metadata": {"tags": ["a", "b"], "count": 5}
            },
            {
                "id": 2,
                "metadata": {"tags": ["c"], "count": 3}
            }
        ]);
        let encoded = encode(&data).unwrap();
        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_hikes_example() {
        let data = json!({
            "context": {
                "task": "Our favorite hikes together",
                "location": "Boulder",
                "season": "spring_2025"
            },
            "friends": ["ana", "luis", "sam"],
            "hikes": [
                {
                    "id": 1,
                    "name": "Blue Lake Trail",
                    "distanceKm": 7.5,
                    "elevationGain": 320,
                    "companion": "ana",
                    "wasSunny": true
                },
                {
                    "id": 2,
                    "name": "Ridge Overlook",
                    "distanceKm": 9.2,
                    "elevationGain": 540,
                    "companion": "luis",
                    "wasSunny": false
                },
                {
                    "id": 3,
                    "name": "Wildflower Loop",
                    "distanceKm": 5.1,
                    "elevationGain": 180,
                    "companion": "sam",
                    "wasSunny": true
                }
            ]
        });

        let encoded = encode(&data).unwrap();
        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded, data);

        // Verify encoded format structure
        assert!(encoded.contains("context.task:"));
        assert!(encoded.contains("friends["));
        assert!(encoded.contains("hikes:@(3):"));
    }

    #[test]
    fn test_string_that_looks_like_number() {
        let data = json!({
            "stringNumber": "123",
            "actualNumber": 123,
            "items": [
                {"id": 1, "code": "001"},
                {"id": 2, "code": "002"}
            ]
        });
        let encoded = encode(&data).unwrap();
        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded, data);
        assert!(decoded["stringNumber"].is_string());
        assert!(decoded["actualNumber"].is_number());
    }

    #[test]
    fn test_string_that_looks_like_boolean() {
        let data = json!({
            "stringTrue": "true",
            "actualTrue": true,
            "stringFalse": "false",
            "actualFalse": false
        });
        let encoded = encode(&data).unwrap();
        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded, data);
        assert!(decoded["stringTrue"].is_string());
        assert!(decoded["actualTrue"].is_boolean());
    }

    #[test]
    fn test_deeply_nested_objects() {
        let data = json!({
            "level1": {
                "level2": {
                    "level3": {
                        "level4": {
                            "value": "deep"
                        }
                    }
                }
            }
        });
        let encoded = encode(&data).unwrap();
        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_boolean_shorthand() {
        let data = json!([
            {"id": 1, "flag": true},
            {"id": 2, "flag": false},
            {"id": 3, "flag": true}
        ]);
        let encoded = encode(&data).unwrap();
        
        // Check that booleans are encoded as T/F
        assert!(encoded.contains(",T") || encoded.contains("T,"));
        assert!(encoded.contains(",F") || encoded.contains("F,"));
        
        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_large_arrays() {
        let items: Vec<serde_json::Value> = (0..100)
            .map(|i| {
                json!({
                    "id": i + 1,
                    "name": format!("Item {}", i + 1),
                    "value": i * 10
                })
            })
            .collect();
        let data = json!({"items": items});
        let encoded = encode(&data).unwrap();
        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_array_of_primitives() {
        let data = json!(["apple", "banana", "cherry"]);
        let encoded = encode(&data).unwrap();
        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded, data);
        assert!(encoded.starts_with('['));
    }
}
