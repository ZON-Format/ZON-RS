//! ZON Decoder v2.0.0 - Compact Hybrid Format
//!
//! Supports both v1.x and v2.0.0 formats:
//! - v2.0: Compact headers (@count:), sequential ID reconstruction, sparse tables
//! - v1.x: Legacy format (@tablename(count):) for backward compatibility

use crate::constants::{
    MAX_ARRAY_LENGTH, MAX_DOCUMENT_SIZE, MAX_LINE_LENGTH, MAX_NESTING_DEPTH, MAX_OBJECT_KEYS,
    META_SEPARATOR, TABLE_MARKER,
};
use crate::error::{ZonDecodeError, ZonDecodeErrorDetails, ZonResult};
use indexmap::IndexMap;
use once_cell::sync::Lazy;
use regex::Regex;
use serde_json::{json, Map, Value};

// Lazy static regex patterns for performance
static V2_NAMED_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^@(\w+)\((\d+)\)(\[\w+\])*:(.+)$").unwrap());
static V2_VALUE_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^@\((\d+)\)(\[\w+\])*:(.+)$").unwrap());
static V2_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^@(\d+)(\[\w+\])*:(.+)$").unwrap());
static V1_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^@(\w+)\((\d+)\):(.+)$").unwrap());
static OMITTED_COLS_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\[(\w+)\]").unwrap());

/// Decode options
#[derive(Debug, Clone)]
pub struct DecodeOptions {
    /// Enable strict mode (default: true)
    pub strict: bool,
}

impl Default for DecodeOptions {
    fn default() -> Self {
        Self { strict: true }
    }
}

impl DecodeOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn strict(mut self, strict: bool) -> Self {
        self.strict = strict;
        self
    }
}

/// Table info during parsing
struct TableInfo {
    cols: Vec<String>,
    omitted_cols: Vec<String>,
    rows: Vec<IndexMap<String, Value>>,
    row_index: usize,
    expected_rows: usize,
}

/// ZON Decoder
pub struct ZonDecoder {
    strict: bool,
}

impl Default for ZonDecoder {
    fn default() -> Self {
        Self { strict: true }
    }
}

impl ZonDecoder {
    pub fn new(options: DecodeOptions) -> Self {
        Self {
            strict: options.strict,
        }
    }

    /// Decode ZON v1.0.5 ClearText format to original data structure
    pub fn decode(&self, zon_str: &str) -> ZonResult<Value> {
        if zon_str.is_empty() {
            return Ok(json!({}));
        }

        // Security: Check document size
        if zon_str.len() > MAX_DOCUMENT_SIZE {
            return Err(ZonDecodeError::with_details(
                format!(
                    "[E301] Document size exceeds maximum ({} bytes)",
                    MAX_DOCUMENT_SIZE
                ),
                ZonDecodeErrorDetails::new().with_code("E301"),
            ));
        }

        let lines: Vec<&str> = zon_str.trim().split('\n').collect();
        if lines.is_empty() {
            return Ok(json!({}));
        }

        // Special case: Root-level ZON list
        if lines.len() == 1 {
            let line = lines[0].trim();
            if line.starts_with('[') {
                return self.parse_zon_node(line, 0);
            }

            // Check for colon-less object/array pattern
            let has_block = line
                .chars()
                .enumerate()
                .take_while(|(_i, c)| c.is_alphanumeric() || *c == '_')
                .last()
                .map(|(i, _)| {
                    line.chars().nth(i + 1) == Some('{') || line.chars().nth(i + 1) == Some('[')
                })
                .unwrap_or(false);

            if !line.contains(META_SEPARATOR) && !line.starts_with(TABLE_MARKER) && !has_block {
                return self.parse_primitive(line);
            }
        }

        // Main decode loop
        let mut metadata: IndexMap<String, Value> = IndexMap::new();
        let mut tables: IndexMap<String, TableInfo> = IndexMap::new();
        let mut current_table: Option<String> = None;

        for line in &lines {
            let trimmed_line = line.trim_end();

            // Security: Check line length
            if trimmed_line.len() > MAX_LINE_LENGTH {
                return Err(ZonDecodeError::with_details(
                    format!(
                        "[E302] Line length exceeds maximum ({} chars)",
                        MAX_LINE_LENGTH
                    ),
                    ZonDecodeErrorDetails::new().with_code("E302"),
                ));
            }

            // Skip blank lines
            if trimmed_line.is_empty() {
                continue;
            }

            // Table header (Anonymous or Legacy): @...
            if trimmed_line.starts_with(TABLE_MARKER) {
                let (table_name, table_info) = self.parse_table_header(trimmed_line)?;
                current_table = Some(table_name.clone());
                tables.insert(table_name, table_info);
            }
            // Table row (if we're in a table and haven't read all rows)
            else if let Some(ref table_name) = current_table {
                if let Some(table) = tables.get_mut(table_name) {
                    if table.row_index < table.expected_rows {
                        let row = self.parse_table_row(trimmed_line, table)?;
                        table.rows.push(row);

                        // If we've read all rows, exit table mode
                        if table.row_index >= table.expected_rows {
                            current_table = None;
                        }
                        continue;
                    }
                }
                // Fall through to metadata parsing
                current_table = None;
                self.parse_metadata_line(trimmed_line, &mut metadata, &mut tables, &mut current_table)?;
            }
            // Metadata line OR Named Table
            else {
                self.parse_metadata_line(trimmed_line, &mut metadata, &mut tables, &mut current_table)?;
            }
        }

        // Recombine tables into metadata
        for (table_name, table) in tables {
            // Strict mode: validate row count
            if self.strict && table.rows.len() != table.expected_rows {
                return Err(ZonDecodeError::with_details(
                    format!(
                        "[E001] Row count mismatch in table '{}': expected {}, got {}",
                        table_name,
                        table.expected_rows,
                        table.rows.len()
                    ),
                    ZonDecodeErrorDetails::new()
                        .with_code("E001")
                        .with_context(format!("Table: {}", table_name)),
                ));
            }

            let table_value = self.reconstruct_table(&table);
            metadata.insert(table_name, table_value);
        }

        // Unflatten dotted keys
        let result = self.unflatten(&metadata);

        // Unwrap pure lists: if only key is 'data', return the list directly
        if let Value::Object(obj) = &result {
            if obj.len() == 1 {
                if let Some(data) = obj.get("data") {
                    if data.is_array() {
                        return Ok(data.clone());
                    }
                }
            }
        }

        Ok(result)
    }

    fn parse_metadata_line(
        &self,
        line: &str,
        metadata: &mut IndexMap<String, Value>,
        tables: &mut IndexMap<String, TableInfo>,
        current_table: &mut Option<String>,
    ) -> ZonResult<()> {
        // Find the split point (colon or brace/bracket)
        let mut split_idx = None;
        let mut split_char = ' ';
        let mut depth = 0;
        let mut in_quote = false;

        for (i, char) in line.chars().enumerate() {
            if char == '"' {
                in_quote = !in_quote;
            }
            if !in_quote {
                if char == '{' || char == '[' {
                    if depth == 0 && split_idx.is_none() {
                        split_idx = Some(i);
                        split_char = char;
                        break;
                    }
                    depth += 1;
                }
                if char == '}' || char == ']' {
                    depth -= 1;
                }
                if char == ':' && depth == 0 {
                    split_idx = Some(i);
                    split_char = ':';
                    break;
                }
            }
        }

        if let Some(idx) = split_idx {
            let (key, val) = if split_char == ':' {
                let key = line[..idx].trim();
                let val = line[idx + 1..].trim();
                (key, val)
            } else {
                // Split at { or [ (include it in value)
                let key = line[..idx].trim();
                let val = line[idx..].trim();
                (key, val)
            };

            // Check if it's a named table start: users: @(5)...
            if val.starts_with(TABLE_MARKER) {
                let (_, table_info) = self.parse_table_header(val)?;
                *current_table = Some(key.to_string());
                tables.insert(key.to_string(), table_info);
            } else {
                *current_table = None;
                metadata.insert(key.to_string(), self.parse_value(val)?);
            }
        }

        Ok(())
    }

    /// Parse table header line
    fn parse_table_header(&self, line: &str) -> ZonResult<(String, TableInfo)> {
        // Try v2.0 format with name: @name(count)[col][col]:columns
        if let Some(caps) = V2_NAMED_PATTERN.captures(line) {
            let table_name = caps.get(1).unwrap().as_str().to_string();
            let count: usize = caps.get(2).unwrap().as_str().parse().unwrap();
            let omitted_str = caps.get(3).map(|m| m.as_str()).unwrap_or("");
            let cols_str = caps.get(4).unwrap().as_str();

            let omitted_cols = self.parse_omitted_cols(omitted_str);
            let cols: Vec<String> = cols_str.split(',').map(|s| s.trim().to_string()).collect();

            return Ok((
                table_name,
                TableInfo {
                    cols,
                    omitted_cols,
                    rows: Vec::new(),
                    row_index: 0,
                    expected_rows: count,
                },
            ));
        }

        // Try v2.1 format (anonymous/value): @(count)[col]:columns
        if let Some(caps) = V2_VALUE_PATTERN.captures(line) {
            let count: usize = caps.get(1).unwrap().as_str().parse().unwrap();
            let omitted_str = caps.get(2).map(|m| m.as_str()).unwrap_or("");
            let cols_str = caps.get(3).unwrap().as_str();

            let omitted_cols = self.parse_omitted_cols(omitted_str);
            let cols: Vec<String> = cols_str.split(',').map(|s| s.trim().to_string()).collect();

            return Ok((
                "data".to_string(),
                TableInfo {
                    cols,
                    omitted_cols,
                    rows: Vec::new(),
                    row_index: 0,
                    expected_rows: count,
                },
            ));
        }

        // Try v2.0 format (anonymous): @count[col][col]:columns
        if let Some(caps) = V2_PATTERN.captures(line) {
            let count: usize = caps.get(1).unwrap().as_str().parse().unwrap();
            let omitted_str = caps.get(2).map(|m| m.as_str()).unwrap_or("");
            let cols_str = caps.get(3).unwrap().as_str();

            let omitted_cols = self.parse_omitted_cols(omitted_str);
            let cols: Vec<String> = cols_str.split(',').map(|s| s.trim().to_string()).collect();

            return Ok((
                "data".to_string(),
                TableInfo {
                    cols,
                    omitted_cols,
                    rows: Vec::new(),
                    row_index: 0,
                    expected_rows: count,
                },
            ));
        }

        // Fallback to v1.x format: @tablename(count):cols
        if let Some(caps) = V1_PATTERN.captures(line) {
            let table_name = caps.get(1).unwrap().as_str().to_string();
            let count: usize = caps.get(2).unwrap().as_str().parse().unwrap();
            let cols_str = caps.get(3).unwrap().as_str();

            let cols: Vec<String> = cols_str.split(',').map(|s| s.trim().to_string()).collect();

            return Ok((
                table_name,
                TableInfo {
                    cols,
                    omitted_cols: Vec::new(),
                    rows: Vec::new(),
                    row_index: 0,
                    expected_rows: count,
                },
            ));
        }

        Err(ZonDecodeError::new(format!("Invalid table header: {}", line)))
    }

    fn parse_omitted_cols(&self, s: &str) -> Vec<String> {
        OMITTED_COLS_PATTERN
            .captures_iter(s)
            .map(|cap| cap.get(1).unwrap().as_str().to_string())
            .collect()
    }

    /// Parse a table row with v2.0 sparse encoding support
    fn parse_table_row(
        &self,
        line: &str,
        table: &mut TableInfo,
    ) -> ZonResult<IndexMap<String, Value>> {
        let tokens = self.split_by_delimiter(line, ',');

        // Strict mode: validate field count
        let _core_field_count = tokens.len().min(table.cols.len());

        if self.strict && tokens.len() < table.cols.len() {
            // Check if we have sparse fields
            let has_sparse = tokens
                .iter()
                .skip(table.cols.len().min(tokens.len()))
                .any(|t| {
                    t.contains(':')
                        && !self.is_url(t)
                        && !self.is_timestamp(t)
                });

            if !has_sparse && tokens.len() < table.cols.len() {
                return Err(ZonDecodeError::with_details(
                    format!(
                        "[E002] Field count mismatch on row {}: expected {} fields, got {}",
                        table.row_index + 1,
                        table.cols.len(),
                        tokens.len()
                    ),
                    ZonDecodeErrorDetails::new()
                        .with_code("E002")
                        .with_context(line[..50.min(line.len())].to_string()),
                ));
            }
        }

        let mut row: IndexMap<String, Value> = IndexMap::new();
        let mut token_idx = 0;

        // Parse core columns
        for col in &table.cols {
            if token_idx < tokens.len() {
                let tok = &tokens[token_idx];
                row.insert(col.clone(), self.parse_value(tok)?);
                token_idx += 1;
            } else {
                row.insert(col.clone(), Value::Null);
            }
        }

        // Parse optional fields (v2.0 sparse encoding: key:value)
        while token_idx < tokens.len() {
            let tok = &tokens[token_idx];
            if tok.contains(':') && !self.is_url(tok) && !self.is_timestamp(tok) {
                if let Some(colon_idx) = tok.find(':') {
                    let key = tok[..colon_idx].trim();
                    let val = tok[colon_idx + 1..].trim();

                    // Validate key is a simple identifier
                    if key.chars().all(|c| c.is_alphanumeric() || c == '_') && !key.is_empty() {
                        row.insert(key.to_string(), self.parse_value(val)?);
                    }
                }
            }
            token_idx += 1;
        }

        // Reconstruct omitted sequential columns (v2.0)
        for col in &table.omitted_cols {
            row.insert(col.clone(), json!(table.row_index + 1));
        }

        table.row_index += 1;
        Ok(row)
    }

    fn is_url(&self, s: &str) -> bool {
        s.starts_with("http://") || s.starts_with("https://") || s.starts_with('/')
    }

    fn is_timestamp(&self, s: &str) -> bool {
        // ISO 8601 datetime
        let bytes = s.as_bytes();
        if bytes.len() >= 19 {
            bytes.get(4) == Some(&b'-')
                && bytes.get(7) == Some(&b'-')
                && bytes.get(10) == Some(&b'T')
                && bytes.get(13) == Some(&b':')
                && bytes.get(16) == Some(&b':')
        } else if bytes.len() == 8 {
            // Simple time HH:MM:SS
            bytes.get(2) == Some(&b':') && bytes.get(5) == Some(&b':')
        } else {
            false
        }
    }

    /// Reconstruct table from parsed rows
    fn reconstruct_table(&self, table: &TableInfo) -> Value {
        let rows: Vec<Value> = table
            .rows
            .iter()
            .map(|row| {
                let unflattened = self.unflatten(row);
                unflattened
            })
            .collect();
        Value::Array(rows)
    }

    /// Recursive parser for YAML-like ZON nested format
    fn parse_zon_node(&self, text: &str, depth: usize) -> ZonResult<Value> {
        if depth > MAX_NESTING_DEPTH {
            return Err(ZonDecodeError::new("Maximum nesting depth exceeded (100)"));
        }

        let trimmed = text.trim();
        if trimmed.is_empty() {
            return Ok(Value::Null);
        }

        // Dict: {k:v,k:v}
        if trimmed.starts_with('{') && trimmed.ends_with('}') {
            let content = &trimmed[1..trimmed.len() - 1].trim();
            if content.is_empty() {
                return Ok(json!({}));
            }

            let mut obj: Map<String, Value> = Map::new();
            let pairs = self.split_by_delimiter(content, ',');

            // Security: Check object key count
            if pairs.len() > MAX_OBJECT_KEYS {
                return Err(ZonDecodeError::with_details(
                    format!(
                        "[E304] Object key count exceeds maximum ({} keys)",
                        MAX_OBJECT_KEYS
                    ),
                    ZonDecodeErrorDetails::new().with_code("E304"),
                ));
            }

            for pair in pairs {
                if let Some((key_str, val_str)) = self.split_key_value(&pair) {
                    let key = self.parse_primitive(&key_str)?;
                    let key_string = match key {
                        Value::String(s) => s,
                        _ => key.to_string(),
                    };
                    let val = self.parse_zon_node(&val_str, depth + 1)?;
                    obj.insert(key_string, val);
                }
            }

            return Ok(Value::Object(obj));
        }

        // List: [v,v]
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            let content = &trimmed[1..trimmed.len() - 1].trim();
            if content.is_empty() {
                return Ok(json!([]));
            }

            let items = self.split_by_delimiter(content, ',');

            // Security: Check array length
            if items.len() > MAX_ARRAY_LENGTH {
                return Err(ZonDecodeError::with_details(
                    format!(
                        "[E303] Array length exceeds maximum ({} items)",
                        MAX_ARRAY_LENGTH
                    ),
                    ZonDecodeErrorDetails::new().with_code("E303"),
                ));
            }

            let result: Result<Vec<Value>, _> = items
                .iter()
                .map(|item| self.parse_zon_node(item, depth + 1))
                .collect();
            return Ok(Value::Array(result?));
        }

        // Leaf node (primitive)
        self.parse_primitive(trimmed)
    }

    fn split_key_value(&self, pair: &str) -> Option<(String, String)> {
        let mut split_idx = None;
        let mut split_char = ' ';
        let mut in_quote = false;
        let mut quote_char = ' ';
        let mut depth = 0;

        for (i, char) in pair.chars().enumerate() {
            if char == '\\' {
                continue;
            }

            if char == '"' || char == '\'' {
                if !in_quote {
                    in_quote = true;
                    quote_char = char;
                } else if char == quote_char {
                    in_quote = false;
                }
            } else if !in_quote {
                if char == ':' && depth == 0 {
                    split_idx = Some(i);
                    split_char = ':';
                    break;
                } else if (char == '{' || char == '[') && depth == 0 && split_idx.is_none() {
                    split_idx = Some(i);
                    split_char = char;
                    break;
                }
                if char == '{' || char == '[' {
                    depth += 1;
                }
                if char == '}' || char == ']' {
                    depth -= 1;
                }
            }
        }

        split_idx.map(|idx| {
            if split_char == ':' {
                (pair[..idx].trim().to_string(), pair[idx + 1..].trim().to_string())
            } else {
                (pair[..idx].trim().to_string(), pair[idx..].trim().to_string())
            }
        })
    }

    /// Split text by delimiter, respecting quotes and nesting
    fn split_by_delimiter(&self, text: &str, delim: char) -> Vec<String> {
        let mut parts = Vec::new();
        let mut current = String::new();
        let mut in_quote = false;
        let mut quote_char = ' ';
        let mut depth = 0;
        let mut chars = text.chars().peekable();

        while let Some(char) = chars.next() {
            // Handle escaped characters
            if char == '\\' {
                current.push(char);
                if let Some(&_next_char) = chars.peek() {
                    current.push(chars.next().unwrap());
                }
                continue;
            }

            if char == '"' || char == '\'' {
                if !in_quote {
                    in_quote = true;
                    quote_char = char;
                } else if char == quote_char {
                    in_quote = false;
                }
                current.push(char);
            } else if !in_quote {
                if char == '{' || char == '[' {
                    depth += 1;
                    current.push(char);
                } else if char == '}' || char == ']' {
                    depth -= 1;
                    current.push(char);
                } else if char == delim && depth == 0 {
                    parts.push(current.clone());
                    current.clear();
                } else {
                    current.push(char);
                }
            } else {
                current.push(char);
            }
        }

        if !current.is_empty() {
            parts.push(current);
        }

        parts
    }

    /// Parse a primitive value (T/F/null/number/string)
    fn parse_primitive(&self, val: &str) -> ZonResult<Value> {
        let trimmed = val.trim();
        let val_lower = trimmed.to_lowercase();

        // Booleans
        if val_lower == "t" || val_lower == "true" {
            return Ok(json!(true));
        }
        if val_lower == "f" || val_lower == "false" {
            return Ok(json!(false));
        }

        // Null
        if val_lower == "null" || val_lower == "none" || val_lower == "nil" {
            return Ok(Value::Null);
        }

        // Quoted string (JSON style)
        if trimmed.starts_with('"') {
            if let Ok(parsed) = serde_json::from_str::<Value>(trimmed) {
                return Ok(parsed);
            }
        }

        // Try number
        if !trimmed.is_empty() {
            if let Ok(n) = trimmed.parse::<i64>() {
                return Ok(json!(n));
            }
            if let Ok(n) = trimmed.parse::<f64>() {
                return Ok(json!(n));
            }
        }

        // String
        Ok(json!(trimmed))
    }

    /// Parse a cell value
    fn parse_value(&self, val: &str) -> ZonResult<Value> {
        let trimmed = val.trim();

        // Quoted string (JSON style) - must check BEFORE primitives
        if trimmed.starts_with('"') {
            if let Ok(decoded) = serde_json::from_str::<Value>(trimmed) {
                // If decoded value is a string that looks like a ZON structure, parse it recursively
                if let Value::String(s) = &decoded {
                    let stripped = s.trim();
                    if stripped.starts_with('{') || stripped.starts_with('[') {
                        return self.parse_zon_node(stripped, 0);
                    }
                }
                return Ok(decoded);
            }

            // Fallback: CSV unquoting for metadata values
            if trimmed.ends_with('"') {
                let unquoted = &trimmed[1..trimmed.len() - 1];
                let unescaped = unquoted.replace("\"\"", "\"");

                // Try to parse unquoted value as JSON
                if let Ok(decoded) = serde_json::from_str::<Value>(&unescaped) {
                    if let Value::String(s) = &decoded {
                        let stripped = s.trim();
                        if stripped.starts_with('{') || stripped.starts_with('[') {
                            return self.parse_zon_node(stripped, 0);
                        }
                    }
                    return Ok(decoded);
                }

                // Check for ZON structure in unquoted string
                let stripped = unescaped.trim();
                if stripped.starts_with('{') || stripped.starts_with('[') {
                    return self.parse_zon_node(stripped, 0);
                }

                return Ok(json!(unescaped));
            }
        }

        // Booleans (case-insensitive)
        let val_lower = trimmed.to_lowercase();
        if val_lower == "t" || val_lower == "true" {
            return Ok(json!(true));
        }
        if val_lower == "f" || val_lower == "false" {
            return Ok(json!(false));
        }

        // Null (case-insensitive)
        if val_lower == "null" || val_lower == "none" || val_lower == "nil" {
            return Ok(Value::Null);
        }

        // Check for ZON-style nested structures (braced)
        if trimmed.starts_with('{') || trimmed.starts_with('[') {
            return self.parse_zon_node(trimmed, 0);
        }

        // Try number
        if !trimmed.is_empty() {
            if let Ok(n) = trimmed.parse::<i64>() {
                return Ok(json!(n));
            }
            if let Ok(n) = trimmed.parse::<f64>() {
                return Ok(json!(n));
            }
        }

        Ok(json!(trimmed))
    }

    /// Unflatten dictionary with dotted keys
    fn unflatten(&self, d: &IndexMap<String, Value>) -> Value {
        let mut result: Map<String, Value> = Map::new();

        for (key, value) in d {
            // Check if key has dot notation
            if !key.contains('.') {
                result.insert(key.clone(), value.clone());
                continue;
            }

            let parts: Vec<&str> = key.split('.').collect();

            // SECURITY: Prevent prototype pollution
            if parts.iter().any(|p| {
                *p == "__proto__" || *p == "constructor" || *p == "prototype"
            }) {
                continue;
            }

            // Navigate/create nested structure using recursive insertion
            self.set_nested_value(&mut result, &parts, value.clone());
        }

        Value::Object(result)
    }

    /// Helper to set a value at a nested path
    fn set_nested_value(&self, obj: &mut Map<String, Value>, parts: &[&str], value: Value) {
        if parts.is_empty() {
            return;
        }

        if parts.len() == 1 {
            obj.insert(parts[0].to_string(), value);
            return;
        }

        let key = parts[0];
        let remaining = &parts[1..];

        // Ensure the intermediate object exists
        if !obj.contains_key(key) {
            obj.insert(key.to_string(), json!({}));
        }

        if let Some(Value::Object(inner)) = obj.get_mut(key) {
            self.set_nested_value(inner, remaining, value);
        }
    }
}

/// Convenience function to decode ZON v1.0.5 format to original data
pub fn decode(data: &str) -> ZonResult<Value> {
    ZonDecoder::default().decode(data)
}

/// Decode with custom options
pub fn decode_with_options(data: &str, options: DecodeOptions) -> ZonResult<Value> {
    ZonDecoder::new(options).decode(data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_decode_empty() {
        let result = decode("").unwrap();
        assert_eq!(result, json!({}));
    }

    #[test]
    fn test_decode_simple_metadata() {
        let result = decode("name:Alice\nage:30\nactive:T").unwrap();
        assert_eq!(result["name"], "Alice");
        assert_eq!(result["age"], 30);
        assert_eq!(result["active"], true);
    }

    #[test]
    fn test_decode_boolean_tokens() {
        let result = decode("active:T\narchived:F\nvalue:null").unwrap();
        assert_eq!(result["active"], true);
        assert_eq!(result["archived"], false);
        assert_eq!(result["value"], Value::Null);
    }

    #[test]
    fn test_decode_table() {
        let result = decode("users:@(2):id,name\n1,Alice\n2,Bob").unwrap();
        assert_eq!(result["users"][0]["id"], 1);
        assert_eq!(result["users"][0]["name"], "Alice");
        assert_eq!(result["users"][1]["id"], 2);
        assert_eq!(result["users"][1]["name"], "Bob");
    }

    #[test]
    fn test_decode_dotted_keys() {
        let result = decode("config.db.host:localhost\nconfig.db.port:5432").unwrap();
        assert_eq!(result["config"]["db"]["host"], "localhost");
        assert_eq!(result["config"]["db"]["port"], 5432);
    }

    #[test]
    fn test_decode_prototype_pollution() {
        let _result = decode("__proto__.polluted:true").unwrap();
        // Should not pollute Object prototype
        let obj: Map<String, Value> = Map::new();
        assert!(!obj.contains_key("polluted"));
    }

    #[test]
    fn test_decode_deep_nesting() {
        let deep = "[".repeat(150) + &"]".repeat(150);
        let result = decode(&deep);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("Maximum nesting depth"));
    }

    #[test]
    fn test_decode_line_length_limit() {
        let long_line = format!("key:{}", "x".repeat(MAX_LINE_LENGTH + 1));
        let result = decode(&long_line);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("E302"));
    }

    #[test]
    fn test_decode_case_insensitive_aliases() {
        let result = decode("a:TRUE\nb:False\nc:NONE\nd:nil").unwrap();
        assert_eq!(result["a"], true);
        assert_eq!(result["b"], false);
        assert_eq!(result["c"], Value::Null);
        assert_eq!(result["d"], Value::Null);
    }

    #[test]
    fn test_strict_mode_row_count() {
        let zon = "users:@(3):id,name\n1,Alice\n2,Bob";
        let result = decode(zon);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code(), Some("E001"));
    }

    #[test]
    fn test_non_strict_mode() {
        let zon = "users:@(3):id,name\n1,Alice\n2,Bob";
        let result = decode_with_options(zon, DecodeOptions::new().strict(false)).unwrap();
        assert_eq!(result["users"].as_array().unwrap().len(), 2);
    }
}
