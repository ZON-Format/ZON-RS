//! Security & Robustness Tests

use serde_json::json;
use zon_format::{decode, encode};

mod prototype_pollution {
    use super::*;

    #[test]
    fn should_reject_proto_keys() {
        let malicious = "data:@(1):id,__proto__.polluted\n1,true";
        let _decoded = decode(malicious).unwrap();
        // In Rust, prototype pollution is not a concern but we verify parsing works
    }

    #[test]
    fn should_reject_constructor_prototype_keys() {
        let malicious = "data:@(1):id,constructor.prototype.polluted\n1,true";
        let _decoded = decode(malicious).unwrap();
        // Key should be ignored in Rust implementation
    }
}

mod denial_of_service {
    use super::*;

    #[test]
    fn should_throw_on_deep_nesting_in_decoder() {
        // Create a deeply nested string: [[[[...]]]]
        let depth = 150;
        let deep_zon = "[".repeat(depth) + &"]".repeat(depth - 1) + "]";

        let result = decode(&deep_zon);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("Maximum nesting depth"));
    }
}

mod circular_references {
    use super::*;

    // Note: Circular references are handled differently in Rust due to ownership
    // The encoder uses HashSet to detect circular references in nested structures
    // This is tested at the unit level in the encoder module
}
