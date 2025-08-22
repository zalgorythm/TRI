//! Sierpinski Triangle Cryptocurrency - Geometric Foundation
//!
//! This library provides the core geometric and mathematical foundation for a cryptocurrency
//! based on the Sierpinski triangle fractal. It includes precise triangle mathematics,
//! fractal generation algorithms, and hierarchical addressing systems.

pub mod core;
pub mod visualization;

// Re-export commonly used types
pub use core::{
    errors::SierpinskiError,
    geometry::Point,
    triangle::Triangle,
    fractal::FractalTriangle,
    state::TriangleState,
    address::TriangleAddress,
    genesis::genesis_triangle,
    subdivision::subdivide_triangle,
    validation::validate_triangle,
};

/// The precision used for all decimal calculations
pub const DECIMAL_PRECISION: u32 = 28;

/// Maximum subdivision depth to prevent infinite recursion
pub const MAX_SUBDIVISION_DEPTH: u8 = 20;

/// Version of the geometric protocol
pub const PROTOCOL_VERSION: &str = "0.1.0";
