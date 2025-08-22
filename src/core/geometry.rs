//! Fundamental geometric types and operations

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::core::errors::{SierpinskiError, SierpinskiResult};

/// A point in 2D space using precise decimal coordinates
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Point {
    pub x: Decimal,
    pub y: Decimal,
}

impl Point {
    /// Create a new point with the given coordinates
    pub fn new(x: Decimal, y: Decimal) -> Self {
        Point { x, y }
    }

    /// Create a point from floating point values (for convenience)
    pub fn from_f64(x: f64, y: f64) -> SierpinskiResult<Self> {
        let x_decimal = Decimal::try_from(x)
            .map_err(|_| SierpinskiError::PrecisionError {
                details: format!("Failed to convert x coordinate: {}", x),
            })?;
        let y_decimal = Decimal::try_from(y)
            .map_err(|_| SierpinskiError::PrecisionError {
                details: format!("Failed to convert y coordinate: {}", y),
            })?;
        Ok(Point::new(x_decimal, y_decimal))
    }

    /// Calculate the distance between two points
    pub fn distance_to(&self, other: &Point) -> SierpinskiResult<Decimal> {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        
        // For precise calculation, we'll use decimal arithmetic
        let distance_squared = dx * dx + dy * dy;
        
        // Simple square root approximation using Newton's method for decimals
        self.decimal_sqrt(distance_squared)
    }

    /// Calculate the midpoint between two points
    pub fn midpoint(&self, other: &Point) -> Point {
        let two = Decimal::from(2);
        Point::new(
            (self.x + other.x) / two,
            (self.y + other.y) / two,
        )
    }

    /// Check if three points are collinear (lie on the same line)
    pub fn are_collinear(p1: &Point, p2: &Point, p3: &Point) -> bool {
        // Calculate the area of the triangle formed by the three points
        // If area is zero, points are collinear
        let area = (p1.x * (p2.y - p3.y) + p2.x * (p3.y - p1.y) + p3.x * (p1.y - p2.y)).abs();
        area < Decimal::new(1, 10) // Very small threshold for floating point comparison
    }

    /// Calculate the cross product of vectors (self->p1) and (self->p2)
    pub fn cross_product(&self, p1: &Point, p2: &Point) -> Decimal {
        let v1x = p1.x - self.x;
        let v1y = p1.y - self.y;
        let v2x = p2.x - self.x;
        let v2y = p2.y - self.y;
        v1x * v2y - v1y * v2x
    }

    /// Simple decimal square root using Newton's method
    pub fn decimal_sqrt(&self, value: Decimal) -> SierpinskiResult<Decimal> {
        if value < Decimal::ZERO {
            return Err(SierpinskiError::ArithmeticOverflow);
        }
        
        if value == Decimal::ZERO {
            return Ok(Decimal::ZERO);
        }

        let mut guess = value / Decimal::from(2);
        let two = Decimal::from(2);
        let precision = Decimal::new(1, 15); // High precision

        for _ in 0..50 { // Maximum iterations
            let new_guess = (guess + value / guess) / two;
            if (new_guess - guess).abs() < precision {
                return Ok(new_guess);
            }
            guess = new_guess;
        }

        Ok(guess)
    }
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

/// Vector operations for geometric calculations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Vector2D {
    pub x: Decimal,
    pub y: Decimal,
}

impl Vector2D {
    /// Create a new vector
    pub fn new(x: Decimal, y: Decimal) -> Self {
        Vector2D { x, y }
    }

    /// Create a vector from two points
    pub fn from_points(from: &Point, to: &Point) -> Self {
        Vector2D::new(to.x - from.x, to.y - from.y)
    }

    /// Calculate the magnitude of the vector
    pub fn magnitude(&self) -> SierpinskiResult<Decimal> {
        let magnitude_squared = self.x * self.x + self.y * self.y;
        Point::new(Decimal::ZERO, Decimal::ZERO).decimal_sqrt(magnitude_squared)
    }

    /// Normalize the vector to unit length
    pub fn normalize(&self) -> SierpinskiResult<Vector2D> {
        let mag = self.magnitude()?;
        if mag == Decimal::ZERO {
            return Err(SierpinskiError::ArithmeticOverflow);
        }
        Ok(Vector2D::new(self.x / mag, self.y / mag))
    }

    /// Calculate dot product with another vector
    pub fn dot(&self, other: &Vector2D) -> Decimal {
        self.x * other.x + self.y * other.y
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_creation() {
        let p = Point::new(Decimal::from(1), Decimal::from(2));
        assert_eq!(p.x, Decimal::from(1));
        assert_eq!(p.y, Decimal::from(2));
    }

    #[test]
    fn test_point_from_f64() {
        let p = Point::from_f64(1.5, 2.5).unwrap();
        assert_eq!(p.x, Decimal::new(15, 1));
        assert_eq!(p.y, Decimal::new(25, 1));
    }

    #[test]
    fn test_midpoint() {
        let p1 = Point::new(Decimal::from(0), Decimal::from(0));
        let p2 = Point::new(Decimal::from(2), Decimal::from(2));
        let mid = p1.midpoint(&p2);
        assert_eq!(mid.x, Decimal::from(1));
        assert_eq!(mid.y, Decimal::from(1));
    }

    #[test]
    fn test_collinear_points() {
        let p1 = Point::new(Decimal::from(0), Decimal::from(0));
        let p2 = Point::new(Decimal::from(1), Decimal::from(1));
        let p3 = Point::new(Decimal::from(2), Decimal::from(2));
        assert!(Point::are_collinear(&p1, &p2, &p3));
    }
}
