//! Triangle data structure and fundamental operations

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::core::geometry::Point;
use crate::core::errors::{SierpinskiError, SierpinskiResult};

/// A triangle defined by three vertices
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Triangle {
    pub vertices: [Point; 3],
}

impl Triangle {
    /// Create a new triangle from three points
    pub fn new(p1: Point, p2: Point, p3: Point) -> SierpinskiResult<Self> {
        let vertices = [p1, p2, p3];
        let triangle = Triangle { vertices };
        
        // Validate that the triangle is not degenerate
        if Point::are_collinear(&p1, &p2, &p3) {
            return Err(SierpinskiError::CollinearPoints);
        }

        if triangle.area()? <= Decimal::ZERO {
            return Err(SierpinskiError::InvalidArea);
        }

        Ok(triangle)
    }

    /// Get the vertices of the triangle
    pub fn vertices(&self) -> &[Point; 3] {
        &self.vertices
    }

    /// Calculate the area of the triangle using the cross product formula
    pub fn area(&self) -> SierpinskiResult<Decimal> {
        let [p1, p2, p3] = self.vertices;
        
        // Area = 0.5 * |cross_product|
        let cross_product = p1.cross_product(&p2, &p3);
        let area = cross_product.abs() / Decimal::from(2);
        
        Ok(area)
    }

    /// Calculate the perimeter of the triangle
    pub fn perimeter(&self) -> SierpinskiResult<Decimal> {
        let [p1, p2, p3] = self.vertices;
        
        let side1 = p1.distance_to(&p2)?;
        let side2 = p2.distance_to(&p3)?;
        let side3 = p3.distance_to(&p1)?;
        
        Ok(side1 + side2 + side3)
    }

    /// Calculate the centroid (center of mass) of the triangle
    pub fn centroid(&self) -> Point {
        let [p1, p2, p3] = self.vertices;
        let three = Decimal::from(3);
        
        Point::new(
            (p1.x + p2.x + p3.x) / three,
            (p1.y + p2.y + p3.y) / three,
        )
    }

    /// Get the three side lengths of the triangle
    pub fn side_lengths(&self) -> SierpinskiResult<[Decimal; 3]> {
        let [p1, p2, p3] = self.vertices;
        
        Ok([
            p1.distance_to(&p2)?,
            p2.distance_to(&p3)?,
            p3.distance_to(&p1)?,
        ])
    }

    /// Check if the triangle is equilateral (all sides equal)
    pub fn is_equilateral(&self) -> SierpinskiResult<bool> {
        let sides = self.side_lengths()?;
        let tolerance = Decimal::new(1, 10); // 0.1 tolerance for floating point comparison
        
        let diff1 = (sides[0] - sides[1]).abs();
        let diff2 = (sides[1] - sides[2]).abs();
        let diff3 = (sides[2] - sides[0]).abs();
        
        Ok(diff1 < tolerance && diff2 < tolerance && diff3 < tolerance)
    }

    /// Check if the triangle is isosceles (two sides equal)
    pub fn is_isosceles(&self) -> SierpinskiResult<bool> {
        let sides = self.side_lengths()?;
        let tolerance = Decimal::new(1, 10);
        
        let eq1 = (sides[0] - sides[1]).abs() < tolerance;
        let eq2 = (sides[1] - sides[2]).abs() < tolerance;
        let eq3 = (sides[2] - sides[0]).abs() < tolerance;
        
        Ok(eq1 || eq2 || eq3)
    }

    /// Get the midpoints of all three sides
    pub fn side_midpoints(&self) -> [Point; 3] {
        let [p1, p2, p3] = self.vertices;
        [
            p1.midpoint(&p2),
            p2.midpoint(&p3),
            p3.midpoint(&p1),
        ]
    }

    /// Check if a point is inside the triangle using barycentric coordinates
    pub fn contains_point(&self, point: &Point) -> bool {
        let [p1, p2, p3] = self.vertices;
        
        // Calculate barycentric coordinates
        let denominator = (p2.y - p3.y) * (p1.x - p3.x) + (p3.x - p2.x) * (p1.y - p3.y);
        
        if denominator == Decimal::ZERO {
            return false; // Degenerate triangle
        }
        
        let a = ((p2.y - p3.y) * (point.x - p3.x) + (p3.x - p2.x) * (point.y - p3.y)) / denominator;
        let b = ((p3.y - p1.y) * (point.x - p3.x) + (p1.x - p3.x) * (point.y - p3.y)) / denominator;
        let c = Decimal::ONE - a - b;
        
        a >= Decimal::ZERO && b >= Decimal::ZERO && c >= Decimal::ZERO
    }

    /// Calculate the scale factor relative to another triangle
    pub fn scale_factor(&self, other: &Triangle) -> SierpinskiResult<Decimal> {
        let my_area = self.area()?;
        let other_area = other.area()?;
        
        if other_area == Decimal::ZERO {
            return Err(SierpinskiError::ArithmeticOverflow);
        }
        
        // Scale factor is sqrt(area_ratio)
        let area_ratio = my_area / other_area;
        Point::new(Decimal::ZERO, Decimal::ZERO).decimal_sqrt(area_ratio)
    }

    /// Generate a unique hash for the triangle based on its vertices
    pub fn hash(&self) -> String {
        let mut hasher = blake3::Hasher::new();
        
        // Hash each vertex coordinate
        for vertex in &self.vertices {
            hasher.update(vertex.x.to_string().as_bytes());
            hasher.update(vertex.y.to_string().as_bytes());
        }
        
        hasher.finalize().to_hex().to_string()
    }
}

impl fmt::Display for Triangle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Triangle[{}, {}, {}]",
            self.vertices[0], self.vertices[1], self.vertices[2]
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_triangle() -> Triangle {
        Triangle::new(
            Point::from_f64(0.0, 0.0).unwrap(),
            Point::from_f64(1.0, 0.0).unwrap(),
            Point::from_f64(0.5, 0.866).unwrap(), // Approximately equilateral
        ).unwrap()
    }

    #[test]
    fn test_triangle_creation() {
        let triangle = create_test_triangle();
        assert_eq!(triangle.vertices.len(), 3);
    }

    #[test]
    fn test_triangle_area() {
        let triangle = create_test_triangle();
        let area = triangle.area().unwrap();
        assert!(area > Decimal::ZERO);
    }

    #[test]
    fn test_triangle_centroid() {
        let triangle = create_test_triangle();
        let centroid = triangle.centroid();
        // Centroid should be approximately (0.5, 0.289)
        assert!((centroid.x - Decimal::new(5, 1)).abs() < Decimal::new(1, 10));
    }

    #[test]
    fn test_collinear_triangle_rejection() {
        let result = Triangle::new(
            Point::from_f64(0.0, 0.0).unwrap(),
            Point::from_f64(1.0, 1.0).unwrap(),
            Point::from_f64(2.0, 2.0).unwrap(),
        );
        assert!(matches!(result, Err(SierpinskiError::CollinearPoints)));
    }

    #[test]
    fn test_triangle_contains_point() {
        let triangle = create_test_triangle();
        let center = triangle.centroid();
        assert!(triangle.contains_point(&center));
        
        let outside_point = Point::from_f64(10.0, 10.0).unwrap();
        assert!(!triangle.contains_point(&outside_point));
    }
}
