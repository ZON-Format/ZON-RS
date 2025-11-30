//! Strict Mode Validation Tests

use serde_json::json;
use zon_format::{decode, decode_with_options, DecodeOptions, ZonDecodeError};

mod e001_row_count_mismatch {
    use super::*;

    #[test]
    fn should_throw_when_table_has_fewer_rows_than_declared_strict_mode() {
        let zon_data = "users:@(3):id,name\n1,Alice\n2,Bob";

        let result = decode(zon_data);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("Row count mismatch"));
        assert_eq!(err.code(), Some("E001"));
    }

    #[test]
    fn should_allow_row_count_mismatch_in_non_strict_mode() {
        let zon_data = "users:@(3):id,name\n1,Alice\n2,Bob";

        let result = decode_with_options(zon_data, DecodeOptions::new().strict(false)).unwrap();
        // Non-strict mode allows fewer rows
        assert_eq!(result["users"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn should_pass_when_row_count_matches_strict_mode() {
        let zon_data = "users:@(2):id,name\n1,Alice\n2,Bob";

        let result = decode(zon_data).unwrap();
        assert_eq!(result["users"].as_array().unwrap().len(), 2);
        assert_eq!(result["users"][0], json!({"id": 1, "name": "Alice"}));
    }
}

mod e002_field_count_mismatch {
    use super::*;

    #[test]
    fn should_throw_when_row_has_fewer_fields_than_declared_columns_strict_mode() {
        let zon_data = "users:@(2):id,name,role\n1,Alice\n2,Bob,admin";

        let result = decode(zon_data);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("Field count mismatch"));
        assert_eq!(err.code(), Some("E002"));
    }

    #[test]
    fn should_allow_missing_fields_in_non_strict_mode() {
        let zon_data = "users:@(2):id,name,role\n1,Alice\n2,Bob,admin";

        let result = decode_with_options(zon_data, DecodeOptions::new().strict(false)).unwrap();
        assert_eq!(result["users"].as_array().unwrap().len(), 2);
        assert_eq!(result["users"][0]["id"], 1);
        assert_eq!(result["users"][0]["name"], "Alice");
        assert_eq!(result["users"][1], json!({"id": 2, "name": "Bob", "role": "admin"}));
    }

    #[test]
    fn should_pass_when_all_rows_have_correct_field_count_strict_mode() {
        let zon_data = "users:@(2):id,name,role\n1,Alice,user\n2,Bob,admin";

        let result = decode(zon_data).unwrap();
        assert_eq!(result["users"].as_array().unwrap().len(), 2);
        assert_eq!(result["users"][0], json!({"id": 1, "name": "Alice", "role": "user"}));
    }

    #[test]
    fn should_allow_sparse_fields_even_in_strict_mode() {
        let zon_data = "users:@(2):id,name\n1,Alice,role:admin,score:98\n2,Bob";

        let result = decode(zon_data).unwrap();
        assert_eq!(result["users"][0], json!({"id": 1, "name": "Alice", "role": "admin", "score": 98}));
        assert_eq!(result["users"][1], json!({"id": 2, "name": "Bob"}));
    }
}

mod error_details {
    use super::*;

    #[test]
    fn should_include_error_code_in_error_object() {
        let zon_data = "users:@(2):id,name\n1,Alice";

        let result = decode(zon_data);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code(), Some("E001"));
    }

    #[test]
    fn should_include_context_in_error_message() {
        let zon_data = "users:@(2):id,name\n1,Alice";

        let result = decode(zon_data);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.context().is_some());
        let display = format!("{}", err);
        assert!(display.contains("Table: users"));
    }
}

mod default_behavior {
    use super::*;

    #[test]
    fn strict_mode_should_be_enabled_by_default() {
        let zon_data = "users:@(2):id,name\n1,Alice";

        // Should throw because default is strict: true
        let result = decode(zon_data);
        assert!(result.is_err());
    }

    #[test]
    fn can_explicitly_enable_strict_mode() {
        let zon_data = "users:@(2):id,name\n1,Alice";

        let result = decode_with_options(zon_data, DecodeOptions::new().strict(true));
        assert!(result.is_err());
    }
}

mod complex_scenarios {
    use super::*;

    #[test]
    fn should_validate_multiple_tables_independently() {
        let zon_data = "users:@(2):id,name\n1,Alice\n2,Bob\nproducts:@(1):id,title\n100,Widget";

        let result = decode(zon_data).unwrap();
        assert_eq!(result["users"].as_array().unwrap().len(), 2);
        assert_eq!(result["products"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn should_work_with_valid_data_across_multiple_tables() {
        let zon_data = "users:@(2):id,name\n1,Alice\n2,Bob\nproducts:@(2):id,title\n100,Widget\n200,Gadget";

        let result = decode(zon_data).unwrap();
        assert_eq!(result["users"].as_array().unwrap().len(), 2);
        assert_eq!(result["products"].as_array().unwrap().len(), 2);
    }
}
