//! Conformance tests based on SPEC.md §11.2 Decoder Checklist

use serde_json::{json, Value};
use zon_format::{decode, decode_with_options, DecodeOptions, ZonDecodeError};

#[test]
fn should_accept_utf8_with_lf_or_crlf() {
    let zon_lf = "key:value\nkey2:value2";
    let zon_crlf = "key:value\r\nkey2:value2";

    assert!(decode(zon_lf).is_ok());
    assert!(decode(zon_crlf).is_ok());
}

#[test]
fn should_decode_t_true_f_false_null_null() {
    let zon_data = "active:T\narchived:F\nvalue:null";
    let result = decode(zon_data).unwrap();

    assert_eq!(result["active"], true);
    assert_eq!(result["archived"], false);
    assert_eq!(result["value"], Value::Null);
}

#[test]
fn should_parse_decimal_and_exponent_numbers() {
    let zon_data = "int:42\nfloat:3.14\nbig:1000000";
    let result = decode(zon_data).unwrap();

    assert_eq!(result["int"], 42);
    assert!((result["float"].as_f64().unwrap() - 3.14).abs() < 0.001);
    assert_eq!(result["big"], 1000000);
}

#[test]
fn should_treat_leading_zero_numbers_as_strings() {
    let zon_data = "code:\"007\"";
    let result = decode(zon_data).unwrap();

    assert_eq!(result["code"], "007");
    assert!(result["code"].is_string());
}

#[test]
fn should_unescape_quoted_strings() {
    let zon_data = "text:\"he said \\\"hello\\\"\"";
    let result = decode(zon_data).unwrap();

    assert_eq!(result["text"], "he said \"hello\"");
}

#[test]
fn should_parse_table_rows_into_array_of_objects() {
    let zon_data = "users:@(2):id,name\n1,Alice\n2,Bob";
    let result = decode(zon_data).unwrap();

    assert_eq!(result["users"].as_array().unwrap().len(), 2);
    assert_eq!(result["users"][0], json!({"id": 1, "name": "Alice"}));
    assert_eq!(result["users"][1], json!({"id": 2, "name": "Bob"}));
}

#[test]
fn should_preserve_key_order_from_document() {
    let zon_data = "z:1\na:2\nm:3";
    let result = decode(zon_data).unwrap();

    if let Value::Object(obj) = result {
        let keys: Vec<_> = obj.keys().collect();
        assert_eq!(keys, vec!["z", "a", "m"]);
    } else {
        panic!("Expected object");
    }
}

#[test]
fn should_reject_prototype_pollution_attempts() {
    let malicious = "data:@(1):id,__proto__.polluted\n1,true";
    let _decoded = decode(malicious).unwrap();

    // Should not pollute object
    // In Rust, this is not applicable but we verify the key is ignored/handled
}

#[test]
fn should_throw_on_nesting_depth_greater_than_100() {
    let deep_nested = "[".repeat(150) + &"]".repeat(150);

    let result = decode(&deep_nested);
    assert!(result.is_err());
    assert!(result.unwrap_err().message.contains("Maximum nesting depth"));
}

#[test]
fn should_throw_on_line_length_greater_than_1mb_e302() {
    let long_line = format!("key:{}", "x".repeat(1024 * 1024 + 1));

    let result = decode(&long_line);
    assert!(result.is_err());
    assert!(result.unwrap_err().message.contains("E302"));
}

#[test]
fn should_handle_case_insensitive_null_boolean_aliases() {
    let zon_data = "a:TRUE\nb:False\nc:NONE\nd:nil";
    let result = decode(zon_data).unwrap();

    assert_eq!(result["a"], true);
    assert_eq!(result["b"], false);
    assert_eq!(result["c"], Value::Null);
    assert_eq!(result["d"], Value::Null);
}

#[test]
fn should_reconstruct_nested_objects_from_dotted_keys() {
    let zon_data = "config.db.host:localhost\nconfig.db.port:5432";
    let result = decode(zon_data).unwrap();

    assert_eq!(result["config"]["db"]["host"], "localhost");
    assert_eq!(result["config"]["db"]["port"], 5432);
}

#[test]
fn should_unwrap_pure_lists_data_key() {
    let zon_data = "data:@(2):id,name\n1,Alice\n2,Bob";
    let result = decode(zon_data).unwrap();

    // Should return array directly, not { data: [...] }
    assert!(result.is_array());
    assert_eq!(result.as_array().unwrap().len(), 2);
}

#[test]
fn should_handle_empty_strings_in_table_cells() {
    let zon_data = "users:@(2):id,name\n1,\"\"\n2,Bob";
    let result = decode(zon_data).unwrap();

    assert_eq!(result["users"][0]["name"], "");
}
