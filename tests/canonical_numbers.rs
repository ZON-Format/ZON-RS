//! Canonical Number Formatting Tests

use serde_json::json;
use zon_format::{decode, encode};

mod integer_numbers {
    use super::*;

    #[test]
    fn should_encode_integers_without_decimal_point() {
        let data = json!({"value": 42});
        let encoded = encode(&data).unwrap();

        assert!(encoded.contains("42"));
        assert!(!encoded.contains("42.0"));
    }

    #[test]
    fn should_handle_zero() {
        let data = json!({"value": 0});
        let encoded = encode(&data).unwrap();

        assert!(encoded.contains("value:0"));
    }

    #[test]
    fn should_handle_negative_integers() {
        let data = json!({"value": -123});
        let encoded = encode(&data).unwrap();

        assert!(encoded.contains("-123"));
    }
}

mod floating_point_numbers {
    use super::*;

    #[test]
    fn should_encode_floats_without_trailing_zeros() {
        let data = json!({"value": 3.14});
        let encoded = encode(&data).unwrap();

        assert!(encoded.contains("3.14"));
        assert!(!encoded.contains("3.140000"));
    }

    #[test]
    fn should_handle_very_small_decimals() {
        let data = json!({"value": 0.001});
        let encoded = encode(&data).unwrap();

        assert!(encoded.contains("0.001"));
        assert!(!encoded.contains("1e-3"));
    }

    #[test]
    fn should_not_use_scientific_notation_for_large_numbers() {
        let data = json!({"value": 1000000});
        let encoded = encode(&data).unwrap();

        assert!(encoded.contains("1000000"));
        assert!(!encoded.contains("1e6"));
        assert!(!encoded.contains("1e+6"));
    }

    #[test]
    fn should_handle_numbers_with_many_decimal_places() {
        let data = json!({"value": 3.141592653589793});
        let encoded = encode(&data).unwrap();

        // Should preserve precision but trim trailing zeros
        assert!(encoded.contains("3.14159265358979"));
        // Should not contain scientific notation
        assert!(!encoded.contains("e+"));
        assert!(!encoded.contains("e-"));
    }
}

mod special_values {
    use super::*;

    #[test]
    fn should_encode_nan_as_null() {
        let data = json!({"value": f64::NAN});
        let encoded = encode(&data).unwrap();

        assert!(encoded.contains("value:null"));
    }

    #[test]
    fn should_encode_infinity_as_null() {
        let data = json!({"value": f64::INFINITY});
        let encoded = encode(&data).unwrap();

        assert!(encoded.contains("value:null"));
    }

    #[test]
    fn should_encode_neg_infinity_as_null() {
        let data = json!({"value": f64::NEG_INFINITY});
        let encoded = encode(&data).unwrap();

        assert!(encoded.contains("value:null"));
    }
}

mod round_trip_preservation {
    use super::*;

    #[test]
    fn should_preserve_integer_values_through_round_trip() {
        let data = json!({"value": 42});
        let encoded = encode(&data).unwrap();
        let decoded = decode(&encoded).unwrap();

        assert_eq!(decoded["value"], 42);
    }

    #[test]
    fn should_preserve_float_values_through_round_trip() {
        let data = json!({"value": 3.14});
        let encoded = encode(&data).unwrap();
        let decoded = decode(&encoded).unwrap();

        assert!((decoded["value"].as_f64().unwrap() - 3.14).abs() < 0.001);
    }

    #[test]
    fn should_preserve_large_numbers_through_round_trip() {
        let data = json!({"value": 1000000});
        let encoded = encode(&data).unwrap();
        let decoded = decode(&encoded).unwrap();

        assert_eq!(decoded["value"], 1000000);
    }

    #[test]
    fn should_preserve_very_small_numbers_through_round_trip() {
        let data = json!({"value": 0.000001});
        let encoded = encode(&data).unwrap();
        let decoded = decode(&encoded).unwrap();

        assert!((decoded["value"].as_f64().unwrap() - 0.000001).abs() < 0.0000001);
    }
}

mod array_of_numbers {
    use super::*;

    #[test]
    fn should_format_all_numbers_canonically_in_arrays() {
        let data = json!({
            "values": [
                {"num": 1000000},
                {"num": 0.001},
                {"num": 42},
                {"num": 3.14}
            ]
        });

        let encoded = encode(&data).unwrap();

        // Should not contain scientific notation (e+, e-, E+, E-)
        assert!(!encoded.contains("e+"));
        assert!(!encoded.contains("e-"));
        assert!(!encoded.contains("E+"));
        assert!(!encoded.contains("E-"));

        // Should contain actual values
        assert!(encoded.contains("1000000"));
        assert!(encoded.contains("0.001"));
        assert!(encoded.contains("42"));
        assert!(encoded.contains("3.14"));
    }
}
