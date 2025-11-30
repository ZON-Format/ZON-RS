//! ZON Error Types
//!
//! This module defines error types for ZON encoding and decoding operations.

use std::fmt;
use thiserror::Error;

/// Error details for ZON decoding errors
#[derive(Debug, Clone, Default)]
pub struct ZonDecodeErrorDetails {
    /// Error code (e.g., "E001", "E002", "E301")
    pub code: Option<String>,
    /// Line number where error occurred
    pub line: Option<usize>,
    /// Column position
    pub column: Option<usize>,
    /// Relevant context snippet
    pub context: Option<String>,
}

impl ZonDecodeErrorDetails {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }

    pub fn with_line(mut self, line: usize) -> Self {
        self.line = Some(line);
        self
    }

    pub fn with_column(mut self, column: usize) -> Self {
        self.column = Some(column);
        self
    }

    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }
}

/// ZON decode error
#[derive(Error, Debug, Clone)]
pub struct ZonDecodeError {
    /// Error message
    pub message: String,
    /// Error details
    pub details: ZonDecodeErrorDetails,
}

impl ZonDecodeError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            details: ZonDecodeErrorDetails::default(),
        }
    }

    pub fn with_details(message: impl Into<String>, details: ZonDecodeErrorDetails) -> Self {
        Self {
            message: message.into(),
            details,
        }
    }

    /// Get the error code
    pub fn code(&self) -> Option<&str> {
        self.details.code.as_deref()
    }

    /// Get the line number
    pub fn line(&self) -> Option<usize> {
        self.details.line
    }

    /// Get the column
    pub fn column(&self) -> Option<usize> {
        self.details.column
    }

    /// Get the context
    pub fn context(&self) -> Option<&str> {
        self.details.context.as_deref()
    }
}

impl fmt::Display for ZonDecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ZonDecodeError")?;
        if let Some(code) = &self.details.code {
            write!(f, " [{}]", code)?;
        }
        write!(f, ": {}", self.message)?;
        if let Some(line) = self.details.line {
            write!(f, " (line {})", line)?;
        }
        if let Some(context) = &self.details.context {
            write!(f, "\n  Context: {}", context)?;
        }
        Ok(())
    }
}

/// ZON encode error
#[derive(Error, Debug, Clone)]
pub enum ZonEncodeError {
    #[error("Circular reference detected")]
    CircularReference,

    #[error("Unsupported type: {0}")]
    UnsupportedType(String),

    #[error("Encoding error: {0}")]
    EncodingError(String),
}

/// Result type for ZON operations
pub type ZonResult<T> = Result<T, ZonDecodeError>;

/// Result type for ZON encode operations  
pub type ZonEncodeResult<T> = Result<T, ZonEncodeError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_error_display() {
        let err = ZonDecodeError::with_details(
            "Row count mismatch",
            ZonDecodeErrorDetails::new()
                .with_code("E001")
                .with_line(5)
                .with_context("Table: users"),
        );

        let display = format!("{}", err);
        assert!(display.contains("E001"));
        assert!(display.contains("Row count mismatch"));
        assert!(display.contains("line 5"));
        assert!(display.contains("Table: users"));
    }

    #[test]
    fn test_decode_error_accessors() {
        let err = ZonDecodeError::with_details(
            "Test error",
            ZonDecodeErrorDetails::new()
                .with_code("E002")
                .with_line(10)
                .with_column(5)
                .with_context("test context"),
        );

        assert_eq!(err.code(), Some("E002"));
        assert_eq!(err.line(), Some(10));
        assert_eq!(err.column(), Some(5));
        assert_eq!(err.context(), Some("test context"));
    }

    #[test]
    fn test_encode_error() {
        let err = ZonEncodeError::CircularReference;
        assert_eq!(format!("{}", err), "Circular reference detected");
    }
}
