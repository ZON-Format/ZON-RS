//! ZON Protocol Constants v1.0.5
//!
//! This module contains all format markers, tokens, and security limits
//! used throughout the ZON format implementation.

/// Format markers

/// Table marker: `@` indicates table structure
pub const TABLE_MARKER: char = '@';

/// Meta separator: `:` separates keys from values
pub const META_SEPARATOR: char = ':';

/// Reserved tokens (for future use)

/// Gas/placeholder token
pub const GAS_TOKEN: &str = "_";

/// Liquid/variable token
pub const LIQUID_TOKEN: &str = "^";

/// Default anchor interval for large datasets
pub const DEFAULT_ANCHOR_INTERVAL: usize = 100;

/// Security limits (DOS prevention)

/// Maximum document size: 100 MB
pub const MAX_DOCUMENT_SIZE: usize = 100 * 1024 * 1024;

/// Maximum line length: 1 MB
pub const MAX_LINE_LENGTH: usize = 1024 * 1024;

/// Maximum array length: 1 million items
pub const MAX_ARRAY_LENGTH: usize = 1_000_000;

/// Maximum object keys: 100K keys
pub const MAX_OBJECT_KEYS: usize = 100_000;

/// Maximum nesting depth: 100 levels
pub const MAX_NESTING_DEPTH: usize = 100;

/// Legacy compatibility (v1.x)

/// Legacy table marker (same as TABLE_MARKER)
pub const LEGACY_TABLE_MARKER: char = '@';

/// Inline threshold rows
pub const INLINE_THRESHOLD_ROWS: usize = 0;

/// Legacy compatibility (kept for potential fallback)

/// Dict reference prefix
pub const DICT_REF_PREFIX: &str = "%";

/// Anchor prefix
pub const ANCHOR_PREFIX: &str = "$";

/// Repeat suffix
pub const REPEAT_SUFFIX: &str = "x";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(TABLE_MARKER, '@');
        assert_eq!(META_SEPARATOR, ':');
        assert_eq!(MAX_DOCUMENT_SIZE, 100 * 1024 * 1024);
        assert_eq!(MAX_LINE_LENGTH, 1024 * 1024);
        assert_eq!(MAX_ARRAY_LENGTH, 1_000_000);
        assert_eq!(MAX_OBJECT_KEYS, 100_000);
        assert_eq!(MAX_NESTING_DEPTH, 100);
    }
}
