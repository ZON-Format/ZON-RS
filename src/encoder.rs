//! ZON Encoder v2.0.0 - Compact Hybrid Format
//!
//! Breaking changes from v1.x:
//! - Compact header syntax (@count: instead of @data(count):)
//! - Sequential ID omission ([col] notation)
//! - Sparse table encoding for semi-uniform data
//! - Adaptive format selection based on data complexity

use crate::constants::{DEFAULT_ANCHOR_INTERVAL, GAS_TOKEN, LIQUID_TOKEN, META_SEPARATOR, TABLE_MARKER};
use crate::error::{ZonEncodeResult};
use indexmap::IndexMap;
use serde_json::Value;
use std::collections::HashSet;

/// ZON Encoder
#[allow(dead_code)]
pub struct ZonEncoder {
    anchor_interval: usize,
}

impl Default for ZonEncoder {
    fn default() -> Self {
        Self::new(DEFAULT_ANCHOR_INTERVAL)
    }
}

impl ZonEncoder {
    /// Create a new encoder with the specified anchor interval
    pub fn new(anchor_interval: usize) -> Self {
        Self { anchor_interval }
    }

    /// Encode data to ZON v1.0.5 ClearText format
    pub fn encode(&self, data: &Value) -> ZonEncodeResult<String> {
        self.encode_internal(data, &mut HashSet::new())
    }

    fn encode_internal(
        &self,
        data: &Value,
        visited: &mut HashSet<usize>,
    ) -> ZonEncodeResult<String> {
        // 1. Root Promotion: Separate metadata from stream
        let (stream_data, metadata, stream_key) = self.extract_primary_stream(data);

        // Fallback for simple/empty data
        if stream_data.is_none()
            && (metadata.is_none() || metadata.as_ref().map_or(true, |m| m.is_empty()))
        {
            if let Value::Object(obj) = data {
                // Special case: Empty object -> empty string
                if obj.is_empty() {
                    return Ok(String::new());
                }
                return self.format_zon_node(data, visited);
            }
            return Ok(serde_json::to_string(data).unwrap_or_default());
        }

        let mut output = Vec::new();

        // Special case: Detect schema uniformity for lists of dicts
        if let Value::Array(arr) = data {
            if !arr.is_empty() && arr.iter().all(|item| matches!(item, Value::Object(_))) {
                // Calculate irregularity score
                let irregularity_score = self.calculate_irregularity(arr);

                // If highly irregular (>60% keys differ), use list format
                if irregularity_score > 0.6 {
                    return self.format_zon_node(data, visited);
                }
            }
        }

        // Determine final stream key
        let final_stream_key = if stream_data.is_some() && stream_key.is_none() {
            Some("data".to_string())
        } else {
            stream_key
        };

        // 3. Write Metadata (YAML-like)
        if let Some(meta) = &metadata {
            if !meta.is_empty() {
                output.extend(self.write_metadata(meta, visited)?);
            }
        }

        // 4. Write Table (if multi-item stream exists)
        if let (Some(stream), Some(key)) = (&stream_data, &final_stream_key) {
            if !output.is_empty() {
                // Add blank line separator
                output.push(String::new());
            }
            output.extend(self.write_table(stream, key, visited)?);
        }

        Ok(output.join("\n"))
    }

    /// Root Promotion Algorithm: Find the main table in the JSON
    fn extract_primary_stream(
        &self,
        data: &Value,
    ) -> (
        Option<Vec<Value>>,
        Option<IndexMap<String, Value>>,
        Option<String>,
    ) {
        match data {
            Value::Array(arr) => {
                // Only promote to table if it contains objects
                if !arr.is_empty() && matches!(arr.first(), Some(Value::Object(_))) {
                    return (Some(arr.clone()), Some(IndexMap::new()), None);
                }

                // v2.0.2 Optimization: Root-level array of primitives
                if !arr.is_empty()
                    && arr
                        .iter()
                        .all(|item| !matches!(item, Value::Object(_) | Value::Array(_)))
                {
                    return (None, Some(IndexMap::new()), None);
                }

                (None, Some(IndexMap::new()), None)
            }
            Value::Object(obj) => {
                // Find largest list of objects
                let mut candidates: Vec<(String, Vec<Value>, usize)> = Vec::new();

                for (k, v) in obj {
                    if let Value::Array(arr) = v {
                        if !arr.is_empty() {
                            // Check if list contains objects (tabular candidate)
                            if matches!(arr.first(), Some(Value::Object(_))) {
                                // Score = Rows * Cols
                                let cols = if let Some(Value::Object(first_obj)) = arr.first() {
                                    first_obj.len()
                                } else {
                                    0
                                };
                                let score = arr.len() * cols;
                                candidates.push((k.clone(), arr.clone(), score));
                            }
                        }
                    }
                }

                if !candidates.is_empty() {
                    candidates.sort_by(|a, b| {
                        b.2.cmp(&a.2).then_with(|| a.0.cmp(&b.0)) // Score desc, then alphabetical
                    });

                    let (key, stream, _) = candidates.into_iter().next().unwrap();
                    let mut meta = IndexMap::new();

                    for (k, v) in obj {
                        if k != &key {
                            meta.insert(k.clone(), v.clone());
                        }
                    }

                    return (Some(stream), Some(meta), Some(key));
                }

                // No table candidates, return as metadata
                let meta: IndexMap<String, Value> =
                    obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
                (None, Some(meta), None)
            }
            _ => (None, None, None),
        }
    }

    /// Write metadata in YAML-like format
    fn write_metadata(
        &self,
        metadata: &IndexMap<String, Value>,
        visited: &mut HashSet<usize>,
    ) -> ZonEncodeResult<Vec<String>> {
        let mut lines = Vec::new();

        // v2.0.3 Optimization: Flatten top-level objects (depth 1)
        let flattened = self.flatten(metadata, "", ".", 1);

        let mut sorted_keys: Vec<_> = flattened.keys().collect();
        sorted_keys.sort();

        for key in sorted_keys {
            let val = &flattened[key];
            let val_str = self.format_value(val, visited)?;

            // v2.0.5 Optimization: Colon-less syntax for root metadata
            if val_str.starts_with('{') || val_str.starts_with('[') {
                lines.push(format!("{}{}", key, val_str));
            } else {
                lines.push(format!("{}{}{}", key, META_SEPARATOR, val_str));
            }
        }

        Ok(lines)
    }

    /// Write table in v2.0.0 compact format with adaptive encoding
    fn write_table(
        &self,
        stream: &[Value],
        key: &str,
        visited: &mut HashSet<usize>,
    ) -> ZonEncodeResult<Vec<String>> {
        if stream.is_empty() {
            return Ok(Vec::new());
        }

        let flat_stream: Vec<IndexMap<String, Value>> =
            stream.iter().map(|row| self.flatten_value(row)).collect();

        // Get all column names
        let mut all_keys: HashSet<String> = HashSet::new();
        for d in &flat_stream {
            all_keys.extend(d.keys().cloned());
        }
        let mut cols: Vec<String> = all_keys.into_iter().collect();
        cols.sort();

        // Analyze column sparsity
        let column_stats = self.analyze_column_sparsity(&flat_stream, &cols);
        let core_columns: Vec<String> = column_stats
            .iter()
            .filter(|(_, presence)| *presence >= 0.7)
            .map(|(name, _)| name.clone())
            .collect();
        let optional_columns: Vec<String> = column_stats
            .iter()
            .filter(|(_, presence)| *presence < 0.7)
            .map(|(name, _)| name.clone())
            .collect();

        // Decide encoding strategy
        let use_sparse_encoding = !optional_columns.is_empty() && optional_columns.len() <= 5;

        if use_sparse_encoding {
            self.write_sparse_table(&flat_stream, &core_columns, &optional_columns, key, visited)
        } else {
            self.write_standard_table(&flat_stream, &cols, key, visited)
        }
    }

    /// Write standard compact table (v2.0.0 format)
    fn write_standard_table(
        &self,
        flat_stream: &[IndexMap<String, Value>],
        cols: &[String],
        key: &str,
        visited: &mut HashSet<usize>,
    ) -> ZonEncodeResult<Vec<String>> {
        let mut lines = Vec::new();
        let row_count = flat_stream.len();

        // Build compact header
        let header = if key != "data" {
            format!(
                "{}{}{}({}){}{}",
                key,
                META_SEPARATOR,
                TABLE_MARKER,
                row_count,
                META_SEPARATOR,
                cols.join(",")
            )
        } else {
            format!("{}{}:{}", TABLE_MARKER, row_count, cols.join(","))
        };
        lines.push(header);

        // Write rows
        for row in flat_stream {
            let tokens: Vec<String> = cols
                .iter()
                .map(|col| {
                    let val = row.get(col).cloned().unwrap_or(Value::Null);
                    self.format_value(&val, visited).unwrap_or_else(|_| "null".to_string())
                })
                .collect();
            lines.push(tokens.join(","));
        }

        Ok(lines)
    }

    /// Write sparse table for semi-uniform data (v2.0.0)
    fn write_sparse_table(
        &self,
        flat_stream: &[IndexMap<String, Value>],
        core_columns: &[String],
        optional_columns: &[String],
        key: &str,
        visited: &mut HashSet<usize>,
    ) -> ZonEncodeResult<Vec<String>> {
        let mut lines = Vec::new();
        let row_count = flat_stream.len();

        // Build header with core columns
        let mut sorted_core: Vec<_> = core_columns.to_vec();
        sorted_core.sort();

        let header = if key != "data" {
            format!(
                "{}{}{}({}){}{}",
                key,
                META_SEPARATOR,
                TABLE_MARKER,
                row_count,
                META_SEPARATOR,
                sorted_core.join(",")
            )
        } else {
            format!("{}{}:{}", TABLE_MARKER, row_count, sorted_core.join(","))
        };
        lines.push(header);

        // Write rows with optional fields
        for row in flat_stream {
            let mut tokens: Vec<String> = sorted_core
                .iter()
                .map(|col| {
                    let val = row.get(col).cloned().unwrap_or(Value::Null);
                    self.format_value(&val, visited).unwrap_or_else(|_| "null".to_string())
                })
                .collect();

            // Append optional columns as key:value if present
            let mut sorted_optional: Vec<_> = optional_columns.to_vec();
            sorted_optional.sort();
            for col in &sorted_optional {
                if let Some(val) = row.get(col) {
                    if !val.is_null() {
                        let formatted = self.format_value(val, visited)?;
                        tokens.push(format!("{}:{}", col, formatted));
                    }
                }
            }

            lines.push(tokens.join(","));
        }

        Ok(lines)
    }

    /// Analyze column sparsity to determine core vs optional
    fn analyze_column_sparsity(
        &self,
        data: &[IndexMap<String, Value>],
        cols: &[String],
    ) -> Vec<(String, f64)> {
        let total = data.len() as f64;
        cols.iter()
            .map(|col| {
                // A column is present if the key exists in the row (even if value is null)
                let presence_count = data
                    .iter()
                    .filter(|row| row.contains_key(col))
                    .count();
                (col.clone(), presence_count as f64 / total)
            })
            .collect()
    }

    /// Calculate irregularity score for array of objects
    fn calculate_irregularity(&self, data: &[Value]) -> f64 {
        if data.is_empty() {
            return 0.0;
        }

        // Get all unique keys across all objects
        let key_sets: Vec<HashSet<String>> = data
            .iter()
            .filter_map(|item| {
                if let Value::Object(obj) = item {
                    Some(obj.keys().cloned().collect())
                } else {
                    None
                }
            })
            .collect();

        if key_sets.len() < 2 {
            return 0.0;
        }

        // Calculate average Jaccard similarity
        let mut total_similarity = 0.0;
        let mut comparisons = 0;

        for i in 0..key_sets.len() {
            for j in (i + 1)..key_sets.len() {
                let shared: usize = key_sets[i].intersection(&key_sets[j]).count();
                let union = key_sets[i].len() + key_sets[j].len() - shared;
                let similarity = if union > 0 {
                    shared as f64 / union as f64
                } else {
                    1.0
                };
                total_similarity += similarity;
                comparisons += 1;
            }
        }

        if comparisons == 0 {
            return 0.0;
        }

        let avg_similarity = total_similarity / comparisons as f64;
        1.0 - avg_similarity
    }

    /// Quote a string for CSV (RFC 4180)
    fn csv_quote(&self, s: &str) -> String {
        let escaped = s.replace('"', "\"\"");
        format!("\"{}\"", escaped)
    }

    /// Format nested structure using YAML-like ZON syntax
    fn format_zon_node(
        &self,
        val: &Value,
        visited: &mut HashSet<usize>,
    ) -> ZonEncodeResult<String> {
        match val {
            Value::Object(obj) => {
                if obj.is_empty() {
                    return Ok("{}".to_string());
                }

                let mut items = Vec::new();
                let mut sorted_keys: Vec<_> = obj.keys().collect();
                sorted_keys.sort();

                for k in sorted_keys {
                    let v = &obj[k];
                    let k_str = if k.chars().any(|c| matches!(c, ',' | ':' | '{' | '}' | '[' | ']' | '"')) {
                        serde_json::to_string(k).unwrap()
                    } else {
                        k.clone()
                    };

                    let v_str = self.format_zon_node(v, visited)?;

                    // v2.0.5 Optimization: Colon-less Objects/Arrays
                    if v_str.starts_with('{') || v_str.starts_with('[') {
                        items.push(format!("{}{}", k_str, v_str));
                    } else {
                        items.push(format!("{}:{}", k_str, v_str));
                    }
                }

                Ok(format!("{{{}}}", items.join(",")))
            }
            Value::Array(arr) => {
                if arr.is_empty() {
                    return Ok("[]".to_string());
                }

                let items: Result<Vec<_>, _> = arr
                    .iter()
                    .map(|item| self.format_zon_node(item, visited))
                    .collect();
                Ok(format!("[{}]", items?.join(",")))
            }
            Value::Null => Ok("null".to_string()),
            Value::Bool(b) => Ok(if *b { "T" } else { "F" }.to_string()),
            Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Ok(i.to_string())
                } else if let Some(f) = n.as_f64() {
                    if !f.is_finite() {
                        return Ok("null".to_string());
                    }
                    let s = format!("{}", f);
                    // Ensure floats have decimal point
                    if !s.contains('.') && !s.contains('e') && !s.contains('E') {
                        Ok(format!("{}.0", s))
                    } else {
                        Ok(s)
                    }
                } else {
                    Ok(n.to_string())
                }
            }
            Value::String(s) => {
                // CRITICAL FIX: Always JSON-stringify strings with newlines
                if s.contains('\n') || s.contains('\r') {
                    return Ok(serde_json::to_string(s).unwrap());
                }

                // ISO Date Detection
                if self.is_iso_date(s) {
                    return Ok(s.clone());
                }

                // Check if needs type protection
                if self.needs_type_protection(s) {
                    return Ok(serde_json::to_string(s).unwrap());
                }

                // Quote empty strings or whitespace-only strings
                if s.trim().is_empty() {
                    return Ok(serde_json::to_string(s).unwrap());
                }

                // Quote if contains structural delimiters
                if s.chars().any(|c| matches!(c, ',' | '{' | '}' | '[' | ']' | '"')) {
                    return Ok(serde_json::to_string(s).unwrap());
                }

                Ok(s.clone())
            }
        }
    }

    /// Format a value with minimal quoting
    fn format_value(
        &self,
        val: &Value,
        visited: &mut HashSet<usize>,
    ) -> ZonEncodeResult<String> {
        match val {
            Value::Null => Ok("null".to_string()),
            Value::Bool(b) => Ok(if *b { "T" } else { "F" }.to_string()),
            Value::Number(n) => {
                if let Some(f) = n.as_f64() {
                    if !f.is_finite() {
                        return Ok("null".to_string());
                    }
                    
                    // Check if it's an integer
                    if let Some(i) = n.as_i64() {
                        return Ok(i.to_string());
                    }
                    
                    // Format float without scientific notation
                    let s = format!("{}", f);
                    
                    // Check for scientific notation and convert
                    if s.contains('e') || s.contains('E') {
                        // Convert scientific notation to fixed point
                        if f.abs() >= 1.0 {
                            Ok(format!("{:.0}", f))
                        } else {
                            // For small numbers, preserve precision
                            let precision = (-f.log10().floor() as usize) + 6;
                            let formatted = format!("{:.prec$}", f, prec = precision.min(15));
                            // Trim trailing zeros but keep at least one decimal place
                            let trimmed = formatted.trim_end_matches('0');
                            if trimmed.ends_with('.') {
                                Ok(format!("{}0", trimmed))
                            } else {
                                Ok(trimmed.to_string())
                            }
                        }
                    } else if !s.contains('.') {
                        Ok(format!("{}.0", s))
                    } else {
                        Ok(s)
                    }
                } else {
                    Ok(n.to_string())
                }
            }
            Value::Array(_) | Value::Object(_) => self.format_zon_node(val, visited),
            Value::String(s) => {
                // CRITICAL FIX: Always JSON-stringify strings with newlines
                if s.contains('\n') || s.contains('\r') {
                    let json_str = serde_json::to_string(s).unwrap();
                    return Ok(self.csv_quote(&json_str));
                }

                // ISO Date Detection
                if self.is_iso_date(s) {
                    return Ok(s.clone());
                }

                // Check if needs type protection
                if self.needs_type_protection(s) {
                    let json_str = serde_json::to_string(s).unwrap();
                    return Ok(self.csv_quote(&json_str));
                }

                // Check if needs CSV quoting
                if self.needs_quotes(s) {
                    return Ok(self.csv_quote(s));
                }

                Ok(s.clone())
            }
        }
    }

    /// Check if string is an ISO 8601 date/datetime
    fn is_iso_date(&self, s: &str) -> bool {
        // ISO 8601 full datetime with timezone
        if s.len() >= 20 {
            let bytes = s.as_bytes();
            if bytes.len() >= 20
                && bytes[4] == b'-'
                && bytes[7] == b'-'
                && bytes[10] == b'T'
                && bytes[13] == b':'
                && bytes[16] == b':'
            {
                // Check for timezone indicator
                if s.ends_with('Z')
                    || (s.len() >= 25 && (s.contains('+') || s[19..].contains('-')))
                {
                    return true;
                }
            }
        }

        // ISO 8601 date only (YYYY-MM-DD)
        if s.len() == 10 {
            let bytes = s.as_bytes();
            if bytes[4] == b'-' && bytes[7] == b'-' {
                return s.chars().enumerate().all(|(i, c)| {
                    if i == 4 || i == 7 {
                        c == '-'
                    } else {
                        c.is_ascii_digit()
                    }
                });
            }
        }

        // Simple time (HH:MM:SS)
        if s.len() == 8 {
            let bytes = s.as_bytes();
            if bytes[2] == b':' && bytes[5] == b':' {
                return s.chars().enumerate().all(|(i, c)| {
                    if i == 2 || i == 5 {
                        c == ':'
                    } else {
                        c.is_ascii_digit()
                    }
                });
            }
        }

        false
    }

    /// Determine if string needs type protection
    fn needs_type_protection(&self, s: &str) -> bool {
        let s_lower = s.to_lowercase();

        // Reserved words
        if matches!(
            s_lower.as_str(),
            "t" | "f" | "true" | "false" | "null" | "none" | "nil"
        ) {
            return true;
        }

        // Gas/Liquid tokens
        if s == GAS_TOKEN || s == LIQUID_TOKEN {
            return true;
        }

        // Leading/trailing whitespace
        if s.trim() != s {
            return true;
        }

        // Control characters
        if s.chars().any(|c| c < '\x20') {
            return true;
        }

        // Pure integer
        if s.chars().all(|c| c.is_ascii_digit() || c == '-') && !s.is_empty() {
            if let Ok(_) = s.parse::<i64>() {
                return true;
            }
        }

        // Pure decimal
        if s.parse::<f64>().is_ok() && s.chars().all(|c| c.is_ascii_digit() || c == '.' || c == '-') {
            return true;
        }

        // Scientific notation
        if s.to_lowercase().contains('e') && s.parse::<f64>().is_ok() {
            return true;
        }

        false
    }

    /// Determine if a string needs quotes
    fn needs_quotes(&self, s: &str) -> bool {
        if s.is_empty() {
            return true;
        }

        // Reserved tokens
        if matches!(s, "T" | "F" | "null") || s == GAS_TOKEN || s == LIQUID_TOKEN {
            return true;
        }

        // Quote if it looks like a number
        if s.parse::<f64>().is_ok() {
            return true;
        }

        // Quote if leading/trailing whitespace
        if s.trim() != s {
            return true;
        }

        // Only quote if contains delimiter or control chars
        if s.chars().any(|c| matches!(c, ',' | '\n' | '\r' | '\t' | '"' | '[' | ']' | '|' | ';')) {
            return true;
        }

        false
    }

    /// Flatten nested dictionary with depth limit
    fn flatten(&self, d: &IndexMap<String, Value>, parent: &str, sep: &str, max_depth: usize) -> IndexMap<String, Value> {
        self.flatten_internal(d, parent, sep, max_depth, 0)
    }

    fn flatten_internal(
        &self,
        d: &IndexMap<String, Value>,
        parent: &str,
        sep: &str,
        max_depth: usize,
        current_depth: usize,
    ) -> IndexMap<String, Value> {
        let mut result = IndexMap::new();

        for (k, v) in d {
            let new_key = if parent.is_empty() {
                k.clone()
            } else {
                format!("{}{}{}", parent, sep, k)
            };

            if let Value::Object(obj) = v {
                if !obj.is_empty() && current_depth < max_depth {
                    let inner: IndexMap<String, Value> =
                        obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
                    let flattened =
                        self.flatten_internal(&inner, &new_key, sep, max_depth, current_depth + 1);
                    result.extend(flattened);
                } else {
                    result.insert(new_key, v.clone());
                }
            } else {
                result.insert(new_key, v.clone());
            }
        }

        result
    }

    fn flatten_value(&self, val: &Value) -> IndexMap<String, Value> {
        match val {
            Value::Object(obj) => {
                let inner: IndexMap<String, Value> =
                    obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
                self.flatten(&inner, "", ".", 0)
            }
            _ => IndexMap::new(),
        }
    }
}

/// Convenience function to encode data to ZON v1.0.5 format
pub fn encode(data: &Value) -> ZonEncodeResult<String> {
    ZonEncoder::default().encode(data)
}

/// Convenience function to encode data with custom anchor interval
pub fn encode_with_interval(data: &Value, anchor_interval: usize) -> ZonEncodeResult<String> {
    ZonEncoder::new(anchor_interval).encode(data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_encode_empty_object() {
        let data = json!({});
        let encoded = encode(&data).unwrap();
        assert_eq!(encoded, "");
    }

    #[test]
    fn test_encode_simple_metadata() {
        let data = json!({
            "name": "Alice",
            "age": 30,
            "active": true
        });
        let encoded = encode(&data).unwrap();
        assert!(encoded.contains("name:Alice"));
        assert!(encoded.contains("age:30"));
        assert!(encoded.contains("active:T"));
    }

    #[test]
    fn test_encode_boolean_shorthand() {
        let data = json!({
            "active": true,
            "archived": false
        });
        let encoded = encode(&data).unwrap();
        assert!(encoded.contains("active:T"));
        assert!(encoded.contains("archived:F"));
        assert!(!encoded.contains("true"));
        assert!(!encoded.contains("false"));
    }

    #[test]
    fn test_encode_null() {
        let data = json!({
            "value": null
        });
        let encoded = encode(&data).unwrap();
        assert!(encoded.contains("value:null"));
    }

    #[test]
    fn test_encode_table() {
        let data = json!({
            "users": [
                {"id": 1, "name": "Alice"},
                {"id": 2, "name": "Bob"}
            ]
        });
        let encoded = encode(&data).unwrap();
        assert!(encoded.contains("users:@(2)"));
        assert!(encoded.contains("id,name"));
    }

    #[test]
    fn test_encode_special_values() {
        let data = json!({
            "nan": f64::NAN,
            "inf": f64::INFINITY,
            "neg_inf": f64::NEG_INFINITY
        });
        let encoded = encode(&data).unwrap();
        assert!(encoded.contains("nan:null"));
        assert!(encoded.contains("inf:null"));
        assert!(encoded.contains("neg_inf:null"));
    }

    #[test]
    fn test_encode_nested_object() {
        let data = json!({
            "config": {
                "database": {
                    "host": "localhost",
                    "port": 5432
                }
            }
        });
        let encoded = encode(&data).unwrap();
        // Should contain flattened or nested structure
        assert!(encoded.contains("config"));
    }

    #[test]
    fn test_iso_date_detection() {
        let encoder = ZonEncoder::default();
        assert!(encoder.is_iso_date("2025-01-01"));
        assert!(encoder.is_iso_date("2025-01-01T10:00:00Z"));
        assert!(encoder.is_iso_date("10:30:00"));
        assert!(!encoder.is_iso_date("hello"));
        assert!(!encoder.is_iso_date("2025-1-1")); // Invalid format
    }
}
