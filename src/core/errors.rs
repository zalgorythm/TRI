//! Comprehensive error handling for geometric operations

use thiserror::Error;

/// Main error type for all Sierpinski triangle operations
#[derive(Error, Debug, Clone, PartialEq)]
pub enum SierpinskiError {
    #[error("Invalid triangle: {reason}")]
    InvalidTriangle { reason: String },

    #[error("Geometric validation failed: {message}")]
    ValidationError { message: String },

    #[error("Subdivision failed: {reason}")]
    SubdivisionError { reason: String },

    #[error("Invalid address format: {address}")]
    InvalidAddress { address: String },

    #[error("Maximum subdivision depth ({max_depth}) exceeded")]
    MaxDepthExceeded { max_depth: u8 },

    #[error("Coordinate precision error: {details}")]
    PrecisionError { details: String },

    #[error("State transition error: from {from} to {to}")]
    StateTransitionError { from: String, to: String },

    #[error("Parent-child relationship error: {reason}")]
    HierarchyError { reason: String },

    #[error("Arithmetic overflow in geometric calculation")]
    ArithmeticOverflow,

    #[error("Invalid triangle configuration: points are collinear")]
    CollinearPoints,

    #[error("Triangle area is zero or negative")]
    InvalidArea,

    #[error("Address path component out of range: {component}")]
    AddressComponentOutOfRange { component: u8 },
}

/// Result type alias for Sierpinski operations
pub type SierpinskiResult<T> = Result<T, SierpinskiError>;

impl SierpinskiError {
    /// Create a validation error with custom message
    pub fn validation(message: impl Into<String>) -> Self {
        SierpinskiError::ValidationError {
            message: message.into(),
        }
    }

    /// Create an invalid triangle error with reason
    pub fn invalid_triangle(reason: impl Into<String>) -> Self {
        SierpinskiError::InvalidTriangle {
            reason: reason.into(),
        }
    }

    /// Create a subdivision error with reason
    pub fn subdivision(reason: impl Into<String>) -> Self {
        SierpinskiError::SubdivisionError {
            reason: reason.into(),
        }
    }
}
