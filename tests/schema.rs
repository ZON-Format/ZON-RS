//! Schema Validation Tests

use serde_json::json;
use zon_format::schema::{zon, validate, validate_value, ZonSchema};

mod primitives {
    use super::*;

    #[test]
    fn should_validate_strings() {
        let schema = zon::string();
        assert!(validate_value(&json!("hello"), &schema).is_success());
        assert!(validate_value(&json!(123), &schema).is_failure());
    }

    #[test]
    fn should_validate_numbers() {
        let schema = zon::number();
        assert!(validate_value(&json!(123), &schema).is_success());
        // '123' string decoded should fail as number
        assert!(validate_value(&json!("123"), &schema).is_failure());
    }

    #[test]
    fn should_validate_booleans() {
        let schema = zon::boolean();
        assert!(validate_value(&json!(true), &schema).is_success());
        // 'true' string should fail as boolean
        assert!(validate_value(&json!("true"), &schema).is_failure());
    }
}

mod enums {
    use super::*;

    #[test]
    fn should_validate_enums() {
        let schema = zon::enum_values(vec!["admin", "user"]);
        assert!(validate_value(&json!("admin"), &schema).is_success());
        assert!(validate_value(&json!("guest"), &schema).is_failure());
    }
}

mod objects {
    use super::*;

    #[test]
    fn should_validate_simple_objects() {
        let schema = zon::object(vec![
            ("name", Box::new(zon::string()) as Box<dyn ZonSchema>),
            ("age", Box::new(zon::number()) as Box<dyn ZonSchema>),
        ]);
        let data = json!({"name": "Alice", "age": 30});
        assert!(validate_value(&data, &schema).is_success());
    }

    #[test]
    fn should_fail_on_missing_keys() {
        let schema = zon::object(vec![
            ("name", Box::new(zon::string()) as Box<dyn ZonSchema>),
            ("age", Box::new(zon::number()) as Box<dyn ZonSchema>),
        ]);
        let data = json!({"name": "Alice"});
        let result = validate_value(&data, &schema);
        assert!(result.is_failure());
        if let zon_format::schema::ZonResult::Failure { error, .. } = result {
            assert!(error.contains("Expected number"));
        }
    }

    #[test]
    fn should_validate_nested_objects() {
        let schema = zon::object(vec![(
            "user",
            Box::new(zon::object(vec![(
                "id",
                Box::new(zon::number()) as Box<dyn ZonSchema>,
            )])) as Box<dyn ZonSchema>,
        )]);
        let data = json!({"user": {"id": 1}});
        assert!(validate_value(&data, &schema).is_success());
    }
}

mod arrays {
    use super::*;

    #[test]
    fn should_validate_arrays_of_primitives() {
        let schema = zon::array(zon::number());
        assert!(validate_value(&json!([1, 2, 3]), &schema).is_success());
        assert!(validate_value(&json!([1, "2"]), &schema).is_failure());
    }

    #[test]
    fn should_validate_arrays_of_objects() {
        let schema = zon::array(zon::object(vec![(
            "id",
            Box::new(zon::number()) as Box<dyn ZonSchema>,
        )]));
        assert!(validate_value(&json!([{"id": 1}, {"id": 2}]), &schema).is_success());
    }
}

mod optional_fields {
    use super::*;

    #[test]
    fn should_handle_optional_fields() {
        let schema = zon::object(vec![
            ("required", Box::new(zon::string()) as Box<dyn ZonSchema>),
            (
                "optional",
                Box::new(zon::string().optional()) as Box<dyn ZonSchema>,
            ),
        ]);

        assert!(validate_value(&json!({"required": "hi"}), &schema).is_success());
        assert!(validate_value(&json!({"required": "hi", "optional": "there"}), &schema).is_success());
        assert!(validate_value(&json!({"required": "hi", "optional": 123}), &schema).is_failure());
    }
}

mod zon_integration {
    use super::*;

    #[test]
    fn should_validate_from_zon_string() {
        let schema = zon::object(vec![
            ("name", Box::new(zon::string()) as Box<dyn ZonSchema>),
            ("active", Box::new(zon::boolean()) as Box<dyn ZonSchema>),
        ]);

        let zon_str = "name:Alice\nactive:T";
        let result = validate(zon_str, &schema);
        assert!(result.is_success());
    }

    #[test]
    fn should_return_useful_error_messages_for_llms() {
        let schema = zon::object(vec![(
            "age",
            Box::new(zon::number()) as Box<dyn ZonSchema>,
        )]);

        let zon_str = "age:\"not a number\"";
        let result = validate(zon_str, &schema);

        assert!(result.is_failure());
        if let zon_format::schema::ZonResult::Failure { error, .. } = result {
            assert!(error.contains("Expected number at age"));
        }
    }
}

mod prompt_generation {
    use super::*;

    #[test]
    fn should_generate_a_prompt_for_a_simple_object() {
        let schema = zon::object(vec![
            (
                "name",
                Box::new(zon::string().describe("Full name")) as Box<dyn ZonSchema>,
            ),
            (
                "age",
                Box::new(zon::number().describe("Age in years")) as Box<dyn ZonSchema>,
            ),
        ]);

        let prompt = schema.to_prompt(0);
        assert!(prompt.contains("name: string - Full name"));
        assert!(prompt.contains("age: number - Age in years"));
    }

    #[test]
    fn should_generate_a_prompt_for_enums() {
        let schema = zon::enum_values(vec!["admin", "user"]).describe("User role");
        let prompt = schema.to_prompt(0);
        assert!(prompt.contains("enum(admin, user) - User role"));
    }

    #[test]
    fn should_generate_a_prompt_for_nested_objects() {
        let schema = zon::object(vec![(
            "user",
            Box::new(
                zon::object(vec![(
                    "id",
                    Box::new(zon::number()) as Box<dyn ZonSchema>,
                )])
                .describe("User details"),
            ) as Box<dyn ZonSchema>,
        )]);

        let prompt = schema.to_prompt(0);
        assert!(prompt.contains("user: object: (User details)"));
        assert!(prompt.contains("id: number"));
    }
}
