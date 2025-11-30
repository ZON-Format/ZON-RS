//! ZON Schema Validation
//!
//! Runtime schema validation library designed for LLM guardrails.
//! Allows defining expected structures and validating LLM outputs against them.

use crate::decoder::decode;
use serde_json::Value;
use std::fmt;

/// Issue found during validation
#[derive(Debug, Clone)]
pub struct ZonIssue {
    /// Path to the issue (e.g., ["users", "0", "name"])
    pub path: Vec<String>,
    /// Error message
    pub message: String,
    /// Error code
    pub code: ZonIssueCode,
}

/// Issue codes for validation errors
#[derive(Debug, Clone, PartialEq)]
pub enum ZonIssueCode {
    InvalidType,
    MissingField,
    InvalidEnum,
    Custom,
}

impl fmt::Display for ZonIssueCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ZonIssueCode::InvalidType => write!(f, "invalid_type"),
            ZonIssueCode::MissingField => write!(f, "missing_field"),
            ZonIssueCode::InvalidEnum => write!(f, "invalid_enum"),
            ZonIssueCode::Custom => write!(f, "custom"),
        }
    }
}

/// Result of schema validation
#[derive(Debug, Clone)]
pub enum ZonResult<T> {
    Success { data: T },
    Failure { error: String, issues: Vec<ZonIssue> },
}

impl<T> ZonResult<T> {
    pub fn is_success(&self) -> bool {
        matches!(self, ZonResult::Success { .. })
    }

    pub fn is_failure(&self) -> bool {
        matches!(self, ZonResult::Failure { .. })
    }

    pub fn data(self) -> Option<T> {
        match self {
            ZonResult::Success { data } => Some(data),
            ZonResult::Failure { .. } => None,
        }
    }

    pub fn error(&self) -> Option<&str> {
        match self {
            ZonResult::Success { .. } => None,
            ZonResult::Failure { error, .. } => Some(error),
        }
    }
}

/// Base trait for all ZON schemas
pub trait ZonSchema: Send + Sync {
    /// Parse and validate data against this schema
    fn parse(&self, data: &Value, path: &[String]) -> ZonResult<Value>;
    
    /// Generate a prompt describing this schema
    fn to_prompt(&self, indent: usize) -> String;
}

/// String schema
pub struct ZonString {
    description: Option<String>,
}

impl ZonString {
    pub fn new() -> Self {
        Self { description: None }
    }

    pub fn describe(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn optional(self) -> ZonOptional<Value> {
        ZonOptional::new(Box::new(self))
    }
}

impl Default for ZonString {
    fn default() -> Self {
        Self::new()
    }
}

impl ZonSchema for ZonString {
    fn parse(&self, data: &Value, path: &[String]) -> ZonResult<Value> {
        if let Value::String(_) = data {
            ZonResult::Success { data: data.clone() }
        } else {
            let path_str = if path.is_empty() {
                "root".to_string()
            } else {
                path.join(".")
            };
            ZonResult::Failure {
                error: format!("Expected string at {}, got {}", path_str, value_type(data)),
                issues: vec![ZonIssue {
                    path: path.to_vec(),
                    message: format!("Expected string, got {}", value_type(data)),
                    code: ZonIssueCode::InvalidType,
                }],
            }
        }
    }

    fn to_prompt(&self, _indent: usize) -> String {
        let desc = self.description.as_ref().map(|d| format!(" - {}", d)).unwrap_or_default();
        format!("string{}", desc)
    }
}

/// Number schema
pub struct ZonNumber {
    description: Option<String>,
}

impl ZonNumber {
    pub fn new() -> Self {
        Self { description: None }
    }

    pub fn describe(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn optional(self) -> ZonOptional<Value> {
        ZonOptional::new(Box::new(self))
    }
}

impl Default for ZonNumber {
    fn default() -> Self {
        Self::new()
    }
}

impl ZonSchema for ZonNumber {
    fn parse(&self, data: &Value, path: &[String]) -> ZonResult<Value> {
        if let Value::Number(n) = data {
            if let Some(f) = n.as_f64() {
                if f.is_nan() {
                    let path_str = if path.is_empty() { "root".to_string() } else { path.join(".") };
                    return ZonResult::Failure {
                        error: format!("Expected number at {}, got NaN", path_str),
                        issues: vec![ZonIssue {
                            path: path.to_vec(),
                            message: "Expected number, got NaN".to_string(),
                            code: ZonIssueCode::InvalidType,
                        }],
                    };
                }
            }
            ZonResult::Success { data: data.clone() }
        } else {
            let path_str = if path.is_empty() { "root".to_string() } else { path.join(".") };
            ZonResult::Failure {
                error: format!("Expected number at {}, got {}", path_str, value_type(data)),
                issues: vec![ZonIssue {
                    path: path.to_vec(),
                    message: format!("Expected number, got {}", value_type(data)),
                    code: ZonIssueCode::InvalidType,
                }],
            }
        }
    }

    fn to_prompt(&self, _indent: usize) -> String {
        let desc = self.description.as_ref().map(|d| format!(" - {}", d)).unwrap_or_default();
        format!("number{}", desc)
    }
}

/// Boolean schema
pub struct ZonBoolean {
    description: Option<String>,
}

impl ZonBoolean {
    pub fn new() -> Self {
        Self { description: None }
    }

    pub fn describe(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn optional(self) -> ZonOptional<Value> {
        ZonOptional::new(Box::new(self))
    }
}

impl Default for ZonBoolean {
    fn default() -> Self {
        Self::new()
    }
}

impl ZonSchema for ZonBoolean {
    fn parse(&self, data: &Value, path: &[String]) -> ZonResult<Value> {
        if let Value::Bool(_) = data {
            ZonResult::Success { data: data.clone() }
        } else {
            let path_str = if path.is_empty() { "root".to_string() } else { path.join(".") };
            ZonResult::Failure {
                error: format!("Expected boolean at {}, got {}", path_str, value_type(data)),
                issues: vec![ZonIssue {
                    path: path.to_vec(),
                    message: format!("Expected boolean, got {}", value_type(data)),
                    code: ZonIssueCode::InvalidType,
                }],
            }
        }
    }

    fn to_prompt(&self, _indent: usize) -> String {
        let desc = self.description.as_ref().map(|d| format!(" - {}", d)).unwrap_or_default();
        format!("boolean{}", desc)
    }
}

/// Enum schema
pub struct ZonEnum {
    values: Vec<String>,
    description: Option<String>,
}

impl ZonEnum {
    pub fn new(values: Vec<String>) -> Self {
        Self {
            values,
            description: None,
        }
    }

    pub fn describe(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn optional(self) -> ZonOptional<Value> {
        ZonOptional::new(Box::new(self))
    }
}

impl ZonSchema for ZonEnum {
    fn parse(&self, data: &Value, path: &[String]) -> ZonResult<Value> {
        if let Value::String(s) = data {
            if self.values.contains(s) {
                return ZonResult::Success { data: data.clone() };
            }
        }
        
        let path_str = if path.is_empty() { "root".to_string() } else { path.join(".") };
        let data_str = match data {
            Value::String(s) => format!("'{}'", s),
            _ => value_type(data).to_string(),
        };
        ZonResult::Failure {
            error: format!(
                "Expected one of [{}] at {}, got {}",
                self.values.join(", "),
                path_str,
                data_str
            ),
            issues: vec![ZonIssue {
                path: path.to_vec(),
                message: format!("Invalid enum value. Expected: {}", self.values.join(", ")),
                code: ZonIssueCode::InvalidEnum,
            }],
        }
    }

    fn to_prompt(&self, _indent: usize) -> String {
        let desc = self.description.as_ref().map(|d| format!(" - {}", d)).unwrap_or_default();
        format!("enum({}){}", self.values.join(", "), desc)
    }
}

/// Array schema
pub struct ZonArray {
    element_schema: Box<dyn ZonSchema>,
    description: Option<String>,
}

impl ZonArray {
    pub fn new(element_schema: Box<dyn ZonSchema>) -> Self {
        Self {
            element_schema,
            description: None,
        }
    }

    pub fn describe(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn optional(self) -> ZonOptional<Value> {
        ZonOptional::new(Box::new(self))
    }
}

impl ZonSchema for ZonArray {
    fn parse(&self, data: &Value, path: &[String]) -> ZonResult<Value> {
        if let Value::Array(arr) = data {
            let mut result = Vec::new();
            for (i, item) in arr.iter().enumerate() {
                let mut item_path = path.to_vec();
                item_path.push(i.to_string());
                match self.element_schema.parse(item, &item_path) {
                    ZonResult::Success { data } => result.push(data),
                    failure => return failure,
                }
            }
            ZonResult::Success { data: Value::Array(result) }
        } else {
            let path_str = if path.is_empty() { "root".to_string() } else { path.join(".") };
            ZonResult::Failure {
                error: format!("Expected array at {}, got {}", path_str, value_type(data)),
                issues: vec![ZonIssue {
                    path: path.to_vec(),
                    message: format!("Expected array, got {}", value_type(data)),
                    code: ZonIssueCode::InvalidType,
                }],
            }
        }
    }

    fn to_prompt(&self, indent: usize) -> String {
        let desc = self.description.as_ref().map(|d| format!(" - {}", d)).unwrap_or_default();
        format!("array of [{}]{}", self.element_schema.to_prompt(indent), desc)
    }
}

/// Object schema
pub struct ZonObject {
    shape: Vec<(String, Box<dyn ZonSchema>)>,
    description: Option<String>,
}

impl ZonObject {
    pub fn new(shape: Vec<(String, Box<dyn ZonSchema>)>) -> Self {
        Self {
            shape,
            description: None,
        }
    }

    pub fn describe(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn optional(self) -> ZonOptional<Value> {
        ZonOptional::new(Box::new(self))
    }
}

impl ZonSchema for ZonObject {
    fn parse(&self, data: &Value, path: &[String]) -> ZonResult<Value> {
        if let Value::Object(obj) = data {
            let mut result = serde_json::Map::new();
            
            for (key, schema) in &self.shape {
                let mut key_path = path.to_vec();
                key_path.push(key.clone());
                
                let value = obj.get(key).cloned().unwrap_or(Value::Null);
                match schema.parse(&value, &key_path) {
                    ZonResult::Success { data } => {
                        result.insert(key.clone(), data);
                    }
                    failure => return failure,
                }
            }
            
            ZonResult::Success { data: Value::Object(result) }
        } else {
            let path_str = if path.is_empty() { "root".to_string() } else { path.join(".") };
            let type_name = if data.is_null() { "null" } else { value_type(data) };
            ZonResult::Failure {
                error: format!("Expected object at {}, got {}", path_str, type_name),
                issues: vec![ZonIssue {
                    path: path.to_vec(),
                    message: format!("Expected object, got {}", type_name),
                    code: ZonIssueCode::InvalidType,
                }],
            }
        }
    }

    fn to_prompt(&self, indent: usize) -> String {
        let spaces = " ".repeat(indent);
        let mut lines = vec!["object:".to_string()];
        
        if let Some(desc) = &self.description {
            lines[0] = format!("object: ({})", desc);
        }
        
        for (key, schema) in &self.shape {
            let field_prompt = schema.to_prompt(indent + 2);
            lines.push(format!("{}  - {}: {}", spaces, key, field_prompt));
        }
        
        lines.join("\n")
    }
}

/// Optional wrapper
pub struct ZonOptional<T> {
    schema: Box<dyn ZonSchema>,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> ZonOptional<T> {
    pub fn new(schema: Box<dyn ZonSchema>) -> Self {
        Self {
            schema,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T> ZonSchema for ZonOptional<T> 
where 
    T: Send + Sync,
{
    fn parse(&self, data: &Value, path: &[String]) -> ZonResult<Value> {
        if data.is_null() {
            return ZonResult::Success { data: Value::Null };
        }
        self.schema.parse(data, path)
    }

    fn to_prompt(&self, indent: usize) -> String {
        format!("{} (optional)", self.schema.to_prompt(indent))
    }
}

/// Schema builder functions
pub mod zon {
    use super::*;

    pub fn string() -> ZonString {
        ZonString::new()
    }

    pub fn number() -> ZonNumber {
        ZonNumber::new()
    }

    pub fn boolean() -> ZonBoolean {
        ZonBoolean::new()
    }

    pub fn enum_values(values: Vec<&str>) -> ZonEnum {
        ZonEnum::new(values.into_iter().map(String::from).collect())
    }

    pub fn array(element_schema: impl ZonSchema + 'static) -> ZonArray {
        ZonArray::new(Box::new(element_schema))
    }

    pub fn object(shape: Vec<(&str, Box<dyn ZonSchema>)>) -> ZonObject {
        ZonObject::new(
            shape
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect(),
        )
    }
}

/// Validates a ZON string or decoded value against a schema
pub fn validate<S: ZonSchema>(input: &str, schema: &S) -> ZonResult<Value> {
    // Try to decode if it looks like ZON
    let data = match decode(input) {
        Ok(v) => v,
        Err(e) => {
            return ZonResult::Failure {
                error: format!("ZON Parse Error: {}", e.message),
                issues: vec![ZonIssue {
                    path: vec![],
                    message: e.message,
                    code: ZonIssueCode::Custom,
                }],
            };
        }
    };

    schema.parse(&data, &[])
}

/// Validates a Value directly against a schema
pub fn validate_value<S: ZonSchema>(data: &Value, schema: &S) -> ZonResult<Value> {
    schema.parse(data, &[])
}

fn value_type(v: &Value) -> &'static str {
    match v {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_string_validation() {
        let schema = zon::string();
        assert!(validate_value(&json!("hello"), &schema).is_success());
        assert!(validate_value(&json!(123), &schema).is_failure());
    }

    #[test]
    fn test_number_validation() {
        let schema = zon::number();
        assert!(validate_value(&json!(123), &schema).is_success());
        assert!(validate_value(&json!(3.14), &schema).is_success());
        assert!(validate_value(&json!("123"), &schema).is_failure());
    }

    #[test]
    fn test_boolean_validation() {
        let schema = zon::boolean();
        assert!(validate_value(&json!(true), &schema).is_success());
        assert!(validate_value(&json!(false), &schema).is_success());
        assert!(validate_value(&json!("true"), &schema).is_failure());
    }

    #[test]
    fn test_enum_validation() {
        let schema = zon::enum_values(vec!["admin", "user"]);
        assert!(validate_value(&json!("admin"), &schema).is_success());
        assert!(validate_value(&json!("guest"), &schema).is_failure());
    }

    #[test]
    fn test_array_validation() {
        let schema = zon::array(zon::number());
        assert!(validate_value(&json!([1, 2, 3]), &schema).is_success());
        assert!(validate_value(&json!([1, "2"]), &schema).is_failure());
    }

    #[test]
    fn test_object_validation() {
        let schema = zon::object(vec![
            ("name", Box::new(zon::string()) as Box<dyn ZonSchema>),
            ("age", Box::new(zon::number()) as Box<dyn ZonSchema>),
        ]);
        
        assert!(validate_value(&json!({"name": "Alice", "age": 30}), &schema).is_success());
        assert!(validate_value(&json!({"name": "Alice"}), &schema).is_failure());
    }

    #[test]
    fn test_optional_field() {
        let schema = zon::object(vec![
            ("required", Box::new(zon::string()) as Box<dyn ZonSchema>),
            ("optional", Box::new(zon::string().optional()) as Box<dyn ZonSchema>),
        ]);
        
        assert!(validate_value(&json!({"required": "hi"}), &schema).is_success());
        assert!(validate_value(&json!({"required": "hi", "optional": "there"}), &schema).is_success());
    }

    #[test]
    fn test_prompt_generation() {
        let schema = zon::object(vec![
            ("name", Box::new(zon::string().describe("Full name")) as Box<dyn ZonSchema>),
            ("age", Box::new(zon::number().describe("Age in years")) as Box<dyn ZonSchema>),
        ]);
        
        let prompt = schema.to_prompt(0);
        assert!(prompt.contains("name: string - Full name"));
        assert!(prompt.contains("age: number - Age in years"));
    }

    #[test]
    fn test_validate_from_zon_string() {
        let schema = zon::object(vec![
            ("name", Box::new(zon::string()) as Box<dyn ZonSchema>),
            ("active", Box::new(zon::boolean()) as Box<dyn ZonSchema>),
        ]);
        
        let zon_str = "name:Alice\nactive:T";
        let result = validate(zon_str, &schema);
        assert!(result.is_success());
    }
}
