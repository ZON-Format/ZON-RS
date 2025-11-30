//! Conformance tests based on SPEC.md §11.1 Encoder Checklist

use serde_json::json;
use zon_format::encode;

#[test]
fn should_emit_utf8_with_lf_line_endings() {
    let data = json!({"a": 1, "b": 2});
    let encoded = encode(&data).unwrap();

    // Should use LF, not CRLF
    assert!(!encoded.contains("\r\n"));
    // Should be a string (UTF-8 compatible)
}

#[test]
fn should_encode_booleans_as_t_f() {
    let data = json!({"active": true, "archived": false});
    let encoded = encode(&data).unwrap();

    assert!(encoded.contains("active:T"));
    assert!(encoded.contains("archived:F"));
    assert!(!encoded.contains("true"));
    assert!(!encoded.contains("false"));
}

#[test]
fn should_encode_null_as_null() {
    let data = json!({"value": null});
    let encoded = encode(&data).unwrap();

    assert!(encoded.contains("value:null"));
}

#[test]
fn should_emit_canonical_numbers() {
    let data = json!({"int": 42, "float": 3.14, "big": 1000000});
    let encoded = encode(&data).unwrap();

    // No scientific notation
    assert!(encoded.contains("1000000"));
    assert!(!encoded.contains("1e6"));
    assert!(!encoded.contains("1e+6"));

    // Has decimal for floats
    assert!(encoded.contains("3.14"));
}

#[test]
fn should_normalize_nan_infinity_to_null() {
    let data = json!({
        "nan": f64::NAN,
        "inf": f64::INFINITY,
        "negInf": f64::NEG_INFINITY
    });
    let encoded = encode(&data).unwrap();

    assert!(encoded.contains("nan:null"));
    assert!(encoded.contains("inf:null"));
    assert!(encoded.contains("negInf:null"));
}

#[test]
fn should_detect_uniform_arrays_as_table_format() {
    let data = json!({
        "users": [
            {"id": 1, "name": "Alice"},
            {"id": 2, "name": "Bob"}
        ]
    });
    let encoded = encode(&data).unwrap();

    // Should have table marker
    assert!(encoded.contains("users:@("));
    assert!(encoded.contains("id,name") || encoded.contains("name,id"));
}

#[test]
fn should_emit_table_headers_with_count_and_columns() {
    let data = json!({
        "items": [
            {"x": 1, "y": 2},
            {"x": 3, "y": 4},
            {"x": 5, "y": 6}
        ]
    });
    let encoded = encode(&data).unwrap();

    assert!(encoded.contains("items:@(3):"));
}

#[test]
fn should_sort_columns_alphabetically() {
    let data = json!({
        "records": [
            {"z": 1, "a": 2, "m": 3}
        ]
    });
    let encoded = encode(&data).unwrap();

    // Columns should be sorted: a, m, z
    assert!(encoded.contains("records:@(1):a,m,z"));
}

#[test]
fn should_quote_strings_with_special_characters() {
    let data = json!({
        "comma": "a,b",
        "colon": "x:y",
        "quote": "say \"hi\""
    });
    let encoded = encode(&data).unwrap();

    assert!(encoded.contains("\"a,b\""));
    // v2.0.5: Colons are allowed unquoted in values
    assert!(encoded.contains("x:y"));
    // Uses quote doubling: " becomes ""
    assert!(encoded.contains("\"say \"\"hi\"\"\""));
}

#[test]
fn should_escape_quotes_in_strings() {
    let data = json!({"text": "he said \"hello\""});
    let encoded = encode(&data).unwrap();

    // Uses quote doubling
    assert!(encoded.contains("\"\"hello\"\""));
}

#[test]
fn should_produce_deterministic_output() {
    let data = json!({"b": 2, "a": 1, "c": 3});

    let encoded1 = encode(&data).unwrap();
    let encoded2 = encode(&data).unwrap();

    assert_eq!(encoded1, encoded2);
}

#[test]
fn should_handle_empty_objects() {
    let data = json!({});
    let encoded = encode(&data).unwrap();

    // Empty object is empty string in ZON
    assert_eq!(encoded, "");
}

#[test]
fn should_handle_empty_arrays() {
    let data = json!({"items": []});
    let encoded = encode(&data).unwrap();

    assert!(!encoded.is_empty());
}
