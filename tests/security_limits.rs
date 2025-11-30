//! Security Limits Tests (DOS Prevention)

use serde_json::json;
use zon_format::{decode, MAX_ARRAY_LENGTH, MAX_DOCUMENT_SIZE, MAX_LINE_LENGTH, MAX_OBJECT_KEYS};

mod e301_document_size_limit {
    use super::*;

    #[test]
    fn should_allow_documents_under_100mb() {
        let doc = "test:value\n".repeat(1000);
        let result = decode(&doc);
        assert!(result.is_ok());
    }

    // Note: Testing actual 100MB+ documents is not practical in unit tests
}

mod e302_line_length_limit {
    use super::*;

    #[test]
    fn should_throw_when_line_exceeds_1mb() {
        let long_line = format!("key:{}", "x".repeat(MAX_LINE_LENGTH + 1));

        let result = decode(&long_line);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("Line length exceeds maximum"));
        assert_eq!(err.code(), Some("E302"));
    }

    #[test]
    fn should_allow_lines_under_1mb() {
        let line = format!("key:{}", "x".repeat(1000));

        let result = decode(&line);
        assert!(result.is_ok());
        assert!(result.unwrap()["key"].is_string());
    }
}

mod e303_array_length_limit {
    use super::*;

    #[test]
    fn should_have_array_length_limit_defined() {
        // The limit exists in implementation at MAX_ARRAY_LENGTH (1M items)
        assert_eq!(MAX_ARRAY_LENGTH, 1_000_000);
    }
}

mod e304_object_key_count_limit {
    use super::*;

    #[test]
    fn should_have_object_key_limit_defined() {
        // The limit exists in implementation at MAX_OBJECT_KEYS (100K keys)
        assert_eq!(MAX_OBJECT_KEYS, 100_000);
    }

    #[test]
    fn should_allow_objects_under_100k_keys() {
        let keys: Vec<String> = (0..100).map(|i| format!("k{}:{}", i, i)).collect();
        let zon_data = format!("data:\"{{{}}}\"", keys.join(","));

        let result = decode(&zon_data);
        assert!(result.is_ok());
    }
}

mod nesting_depth_limit {
    use super::*;

    #[test]
    fn should_throw_when_nesting_exceeds_100_levels() {
        let nested = "[".repeat(150) + &"]".repeat(150);

        let result = decode(&nested);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("Maximum nesting depth"));
    }

    #[test]
    fn should_allow_nesting_under_100_levels() {
        let nested = "[".repeat(50) + &"]".repeat(50);

        let result = decode(&nested);
        assert!(result.is_ok());
    }
}

mod combined_limits {
    use super::*;

    #[test]
    fn should_work_with_normal_data_within_all_limits() {
        let zon_data = r#"
metadata:"{version:1.0.5,env:prod}"
users:@(3):id,name
1,Alice
2,Bob
3,Carol
tags:"[nodejs,typescript,llm]"
"#;

        let result = decode(zon_data).unwrap();
        assert_eq!(result["users"].as_array().unwrap().len(), 3);
        assert_eq!(result["metadata"]["version"], "1.0.5");
        assert_eq!(result["tags"].as_array().unwrap().len(), 3);
    }
}
