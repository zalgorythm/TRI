//! Core geometric and mathematical components

pub mod errors;
pub mod geometry;
pub mod triangle;
pub mod fractal;
pub mod genesis;
pub mod subdivision;
pub mod address;
pub mod validation;
pub mod state;
pub mod block;
pub mod blockchain;
pub mod mining;
pub mod wallet;
pub mod network;
pub mod economics;

// Re-export all core types
pub use errors::*;
pub use geometry::*;
pub use triangle::*;
pub use fractal::*;
pub use genesis::*;
pub use subdivision::*;
pub use address::*;
pub use validation::*;
pub use state::*;
