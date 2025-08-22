//! Genesis triangle creation and management

use rust_decimal::Decimal;
use crate::core::{
    geometry::Point,
    triangle::Triangle,
    fractal::FractalTriangle,
    errors::SierpinskiResult,
};

/// Create the perfect equilateral genesis triangle
pub fn genesis_triangle() -> SierpinskiResult<Triangle> {
    // Create a perfect equilateral triangle with side length 1
    // Centered at origin with one vertex pointing up
    
    let side_length = Decimal::ONE;
    let height = side_length * Decimal::new(866, 3); // sqrt(3)/2 ≈ 0.866
    let half_side = side_length / Decimal::from(2);
    
    let bottom_left = Point::new(-half_side, -height / Decimal::from(3));
    let bottom_right = Point::new(half_side, -height / Decimal::from(3));
    let top = Point::new(Decimal::ZERO, height * Decimal::from(2) / Decimal::from(3));
    
    Triangle::new(bottom_left, bottom_right, top)
}

/// Create the genesis fractal triangle
pub fn genesis_fractal_triangle() -> SierpinskiResult<FractalTriangle> {
    let triangle = genesis_triangle()?;
    Ok(FractalTriangle::genesis(triangle))
}

/// Alternative genesis triangle with custom size and position
pub fn genesis_triangle_with_size(
    center: Point,
    side_length: Decimal,
) -> SierpinskiResult<Triangle> {
    let height = side_length * Decimal::new(866, 3); // sqrt(3)/2
    let half_side = side_length / Decimal::from(2);
    let third_height = height / Decimal::from(3);
    
    let bottom_left = Point::new(
        center.x - half_side,
        center.y - third_height,
    );
    let bottom_right = Point::new(
        center.x + half_side,
        center.y - third_height,
    );
    let top = Point::new(
        center.x,
        center.y + height * Decimal::from(2) / Decimal::from(3),
    );
    
    Triangle::new(bottom_left, bottom_right, top)
}

/// Create a genesis triangle that fits within specified bounds
pub fn genesis_triangle_bounded(
    min_x: Decimal,
    max_x: Decimal,
    min_y: Decimal,
    max_y: Decimal,
) -> SierpinskiResult<Triangle> {
    let width = max_x - min_x;
    let height = max_y - min_y;
    
    // Calculate the maximum side length that fits
    let max_side_from_width = width;
    let max_side_from_height = height * Decimal::from(2) / Decimal::new(866, 3); // height / (sqrt(3)/2)
    
    let side_length = if max_side_from_width < max_side_from_height {
        max_side_from_width
    } else {
        max_side_from_height
    } * Decimal::new(9, 1); // 90% to add some margin
    
    let center = Point::new(
        (min_x + max_x) / Decimal::from(2),
        (min_y + max_y) / Decimal::from(2),
    );
    
    genesis_triangle_with_size(center, side_length)
}

/// Validate that a triangle is suitable as a genesis triangle
pub fn validate_genesis_triangle(triangle: &Triangle) -> SierpinskiResult<bool> {
    // Check if triangle is equilateral
    let is_equilateral = triangle.is_equilateral()?;
    if !is_equilateral {
        return Ok(false);
    }
    
    // Check if triangle has positive area
    let area = triangle.area()?;
    if area <= Decimal::ZERO {
        return Ok(false);
    }
    
    // Check if triangle is properly oriented (no flipped triangles)
    let vertices = triangle.vertices();
    let cross_product = vertices[0].cross_product(&vertices[1], &vertices[2]);
    if cross_product <= Decimal::ZERO {
        return Ok(false); // Triangle is clockwise or degenerate
    }
    
    Ok(true)
}

/// Calculate the theoretical maximum subdivision depth for a triangle
pub fn max_theoretical_depth(triangle: &Triangle) -> SierpinskiResult<u8> {
    let area = triangle.area()?;
    
    // Each subdivision reduces area by factor of 3/4
    // We stop when area becomes smaller than minimum representable decimal
    let min_area = Decimal::new(1, 28); // Smallest representable area
    let reduction_factor = Decimal::new(3, 0) / Decimal::new(4, 0);
    
    let mut current_area = area;
    let mut depth = 0u8;
    
    while current_area > min_area && depth < crate::MAX_SUBDIVISION_DEPTH {
        current_area *= reduction_factor;
        depth += 1;
    }
    
    Ok(depth)
}

/// Genesis triangle properties for mathematical verification
#[derive(Debug, Clone)]
pub struct GenesisProperties {
    pub side_length: Decimal,
    pub area: Decimal,
    pub perimeter: Decimal,
    pub centroid: Point,
    pub is_equilateral: bool,
    pub max_depth: u8,
}

impl GenesisProperties {
    /// Calculate properties of a genesis triangle
    pub fn calculate(triangle: &Triangle) -> SierpinskiResult<Self> {
        let side_lengths = triangle.side_lengths()?;
        let side_length = side_lengths[0]; // All sides should be equal
        let area = triangle.area()?;
        let perimeter = triangle.perimeter()?;
        let centroid = triangle.centroid();
        let is_equilateral = triangle.is_equilateral()?;
        let max_depth = max_theoretical_depth(triangle)?;
        
        Ok(GenesisProperties {
            side_length,
            area,
            perimeter,
            centroid,
            is_equilateral,
            max_depth,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_genesis_triangle_creation() {
        let triangle = genesis_triangle().unwrap();
        assert!(triangle.is_equilateral().unwrap());
        assert!(triangle.area().unwrap() > Decimal::ZERO);
    }

    #[test]
    fn test_genesis_validation() {
        let triangle = genesis_triangle().unwrap();
        assert!(validate_genesis_triangle(&triangle).unwrap());
    }

    #[test]
    fn test_genesis_with_custom_size() {
        let center = Point::new(Decimal::from(5), Decimal::from(5));
        let side_length = Decimal::from(2);
        
        let triangle = genesis_triangle_with_size(center, side_length).unwrap();
        assert!(triangle.is_equilateral().unwrap());
        
        let centroid = triangle.centroid();
        // Centroid should be close to the specified center
        let tolerance = Decimal::new(1, 2); // 0.01
        assert!((centroid.x - center.x).abs() < tolerance);
        assert!((centroid.y - center.y).abs() < tolerance);
    }

    #[test]
    fn test_bounded_genesis() {
        let min_x = Decimal::from(-10);
        let max_x = Decimal::from(10);
        let min_y = Decimal::from(-10);
        let max_y = Decimal::from(10);
        
        let triangle = genesis_triangle_bounded(min_x, max_x, min_y, max_y).unwrap();
        
        // Check that all vertices are within bounds
        for vertex in triangle.vertices() {
            assert!(vertex.x >= min_x && vertex.x <= max_x);
            assert!(vertex.y >= min_y && vertex.y <= max_y);
        }
    }

    #[test]
    fn test_genesis_properties() {
        let triangle = genesis_triangle().unwrap();
        let properties = GenesisProperties::calculate(&triangle).unwrap();
        
        assert!(properties.is_equilateral);
        assert!(properties.area > Decimal::ZERO);
        assert!(properties.perimeter > Decimal::ZERO);
        assert!(properties.max_depth > 0);
    }
}
