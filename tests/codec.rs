//! ZON Codec Tests
//! Port of test_codec.ts from the TypeScript implementation

use pretty_assertions::assert_eq;
use serde_json::json;
use zon_format::{decode, encode};

#[test]
fn test_empty_object() {
    let data = json!({});
    let encoded = encode(&data).unwrap();
    let decoded = decode(&encoded).unwrap();
    assert_eq!(decoded, data);
}

#[test]
fn test_simple_metadata() {
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
fn test_nested_object() {
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
fn test_array_of_objects_table() {
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
fn test_mixed_metadata_and_table() {
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
fn test_boolean_values() {
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
fn test_null_values() {
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
fn test_numbers_integers_and_floats() {
    let data = json!({
        "integer": 42,
        "float": 3.14,
        "negative": -10,
        "negativeFloat": -2.5,
        "items": [
            {"id": 1, "value": 100},
            {"id": 2, "value": 200.5}
        ]
    });
    let encoded = encode(&data).unwrap();
    let decoded = decode(&encoded).unwrap();
    
    // Compare individual fields due to float precision
    assert_eq!(decoded["integer"], data["integer"]);
    assert_eq!(decoded["negative"], data["negative"]);
    assert!((decoded["float"].as_f64().unwrap() - 3.14).abs() < 0.001);
    assert!((decoded["negativeFloat"].as_f64().unwrap() - (-2.5)).abs() < 0.001);
}

#[test]
fn test_strings_with_special_characters() {
    let data = json!({
        "plain": "hello",
        "withComma": "hello, world",
        "withQuotes": "say \"hello\"",
        "withNewline": "line1\nline2",
        "items": [
            {"id": 1, "text": "normal"},
            {"id": 2, "text": "with, comma"}
        ]
    });
    let encoded = encode(&data).unwrap();
    let decoded = decode(&encoded).unwrap();
    assert_eq!(decoded, data);
}

#[test]
fn test_empty_arrays() {
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
fn test_nested_arrays_in_metadata() {
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
fn test_complex_nested_objects_in_table_cells() {
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
fn test_hikes_example_from_readme() {
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

    // Verify the encoded format structure
    assert!(encoded.contains("context.task:"));
    assert!(encoded.contains("friends["));
    assert!(encoded.contains("hikes:@(3):companion,distanceKm,elevationGain,id,name,wasSunny"));
}

// Edge cases

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
        "actualFalse": false,
        "items": [
            {"id": 1, "status": "T"},
            {"id": 2, "status": true}
        ]
    });
    let encoded = encode(&data).unwrap();
    let decoded = decode(&encoded).unwrap();
    assert_eq!(decoded, data);
    assert!(decoded["stringTrue"].is_string());
    assert!(decoded["actualTrue"].is_boolean());
}

#[test]
fn test_empty_strings() {
    let data = json!({
        "empty": "",
        "items": [
            {"id": 1, "name": ""},
            {"id": 2, "name": "value"}
        ]
    });
    let encoded = encode(&data).unwrap();
    let decoded = decode(&encoded).unwrap();
    assert_eq!(decoded, data);
}

#[test]
fn test_whitespace_preservation() {
    let data = json!({
        "leading": "  space",
        "trailing": "space  ",
        "both": "  both  ",
        "items": [
            {"id": 1, "text": "  padded  "}
        ]
    });
    let encoded = encode(&data).unwrap();
    let decoded = decode(&encoded).unwrap();
    assert_eq!(decoded, data);
}

#[test]
fn test_very_long_strings() {
    let long_string = "a".repeat(1000);
    let data = json!({
        "long": long_string,
        "items": [
            {"id": 1, "text": long_string}
        ]
    });
    let encoded = encode(&data).unwrap();
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
    // Should be encoded as JSON array, not table
    assert!(encoded.starts_with('['));
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

// Data type preservation tests

#[test]
fn test_integer_vs_float_distinction() {
    let data = json!({
        "integer": 42,
        "float": 42.0,
        "explicitFloat": 3.14,
        "items": [
            {"id": 1, "intVal": 100, "floatVal": 100.5},
            {"id": 2, "intVal": 200, "floatVal": 200.0}
        ]
    });
    let encoded = encode(&data).unwrap();
    let decoded = decode(&encoded).unwrap();

    assert_eq!(decoded["integer"], 42);
    assert!((decoded["explicitFloat"].as_f64().unwrap() - 3.14).abs() < 0.001);
}

#[test]
fn test_boolean_shorthand_tf() {
    let data = json!([
        {"id": 1, "flag": true},
        {"id": 2, "flag": false},
        {"id": 3, "flag": true}
    ]);
    let encoded = encode(&data).unwrap();

    // Check that booleans are encoded as T/F
    assert!(encoded.contains(",T") || encoded.contains("T,") || encoded.contains("\nT"));
    assert!(encoded.contains(",F") || encoded.contains("F,") || encoded.contains("\nF"));

    let decoded = decode(&encoded).unwrap();
    assert_eq!(decoded, data);
}
