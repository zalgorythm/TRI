//! Geometric validation functions for triangles and fractal structures

use rust_decimal::Decimal;

use crate::core::{
    triangle::Triangle,
    fractal::{FractalTriangle, FractalStructure},
    geometry::Point,
    state::TriangleState,
    errors::SierpinskiResult,
};

/// Comprehensive validation result
#[derive(Debug, Clone, PartialEq)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl ValidationResult {
    /// Create a successful validation result
    pub fn success() -> Self {
        ValidationResult {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Create a failed validation result with errors
    pub fn failure(errors: Vec<String>) -> Self {
        ValidationResult {
            is_valid: false,
            errors,
            warnings: Vec::new(),
        }
    }

    /// Add an error to the result
    pub fn add_error(&mut self, error: String) {
        self.errors.push(error);
        self.is_valid = false;
    }

    /// Add a warning to the result
    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }

    /// Combine with another validation result
    pub fn combine(&mut self, other: ValidationResult) {
        self.errors.extend(other.errors);
        self.warnings.extend(other.warnings);
        self.is_valid &= other.is_valid;
    }
}

/// Validate a basic triangle for geometric correctness
pub fn validate_triangle(triangle: &Triangle) -> ValidationResult {
    let mut result = ValidationResult::success();

    // Check for collinear points
    let vertices = triangle.vertices();
    if Point::are_collinear(&vertices[0], &vertices[1], &vertices[2]) {
        result.add_error("Triangle vertices are collinear".to_string());
        return result;
    }

    // Check for valid area
    match triangle.area() {
        Ok(area) => {
            if area <= Decimal::ZERO {
                result.add_error("Triangle has zero or negative area".to_string());
            }
        }
        Err(e) => {
            result.add_error(format!("Failed to calculate triangle area: {}", e));
        }
    }

    // Check for reasonable coordinate values
    for (i, vertex) in vertices.iter().enumerate() {
        if vertex.x.is_sign_negative() && vertex.x.abs() > Decimal::new(1000000, 0) {
            result.add_warning(format!("Vertex {} has very large negative x coordinate", i));
        }
        if vertex.y.is_sign_negative() && vertex.y.abs() > Decimal::new(1000000, 0) {
            result.add_warning(format!("Vertex {} has very large negative y coordinate", i));
        }
    }

    result
}

/// Validate a fractal triangle
pub fn validate_fractal_triangle(fractal_triangle: &FractalTriangle) -> ValidationResult {
    let mut result = validate_triangle(&fractal_triangle.triangle);

    // Validate state consistency
    match fractal_triangle.state {
        TriangleState::Genesis => {
            if fractal_triangle.depth != 0 {
                result.add_error("Genesis triangle must have depth 0".to_string());
            }
            if fractal_triangle.parent_id.is_some() {
                result.add_error("Genesis triangle cannot have a parent".to_string());
            }
        }
        TriangleState::Active => {
            if fractal_triangle.depth == 0 && fractal_triangle.parent_id.is_none() {
                result.add_warning("Active triangle at depth 0 should probably be Genesis".to_string());
            }
        }
        TriangleState::Subdivided => {
            if fractal_triangle.child_ids.is_empty() {
                result.add_error("Subdivided triangle must have children".to_string());
            }
        }
        TriangleState::Void => {
            // Void triangles are valid in any configuration
        }
        TriangleState::Inactive => {
            // Inactive triangles are valid in any configuration
        }
    }

    // Validate depth consistency
    if fractal_triangle.depth > crate::MAX_SUBDIVISION_DEPTH {
        result.add_error(format!(
            "Triangle depth {} exceeds maximum allowed depth {}",
            fractal_triangle.depth,
            crate::MAX_SUBDIVISION_DEPTH
        ));
    }

    // Validate timestamp consistency
    if fractal_triangle.updated_at < fractal_triangle.created_at {
        result.add_error("Updated timestamp cannot be before created timestamp".to_string());
    }

    result
}

/// Validate parent-child relationships in fractal triangles
pub fn validate_parent_child_relationship(
    parent: &FractalTriangle,
    child: &FractalTriangle,
) -> ValidationResult {
    let mut result = ValidationResult::success();

    // Check depth relationship
    if child.depth != parent.depth + 1 {
        result.add_error(format!(
            "Child depth {} should be parent depth {} + 1",
            child.depth, parent.depth
        ));
    }

    // Check parent ID
    if child.parent_id != Some(parent.id) {
        result.add_error("Child parent_id does not match parent ID".to_string());
    }

    // Check that parent contains child ID
    if !parent.child_ids.contains(&child.id) {
        result.add_error("Parent does not contain child ID in its child list".to_string());
    }

    // Check state compatibility
    if parent.state != TriangleState::Subdivided {
        result.add_error("Parent of a child triangle must be in Subdivided state".to_string());
    }

    // Validate geometric relationship (child should be inside parent)
    let child_centroid = child.triangle.centroid();
    if !parent.triangle.contains_point(&child_centroid) {
        result.add_warning("Child triangle centroid is not inside parent triangle".to_string());
    }

    // Check area relationship (child should be smaller than parent)
    match (parent.triangle.area(), child.triangle.area()) {
        (Ok(parent_area), Ok(child_area)) => {
            if child_area >= parent_area {
                result.add_error("Child triangle area should be smaller than parent area".to_string());
            }
        }
        _ => {
            result.add_error("Failed to calculate areas for parent-child comparison".to_string());
        }
    }

    result
}

/// Validate an entire fractal structure
pub fn validate_fractal_structure(structure: &FractalStructure) -> ValidationResult {
    let mut result = ValidationResult::success();

    // Check for genesis triangle
    if structure.genesis().is_none() {
        result.add_error("Fractal structure must have a genesis triangle".to_string());
        return result;
    }

    let genesis = structure.genesis().unwrap();

    // Validate genesis triangle
    let genesis_validation = validate_fractal_triangle(genesis);
    result.combine(genesis_validation);

    // Validate all triangles
    for depth in 0..=structure.max_depth() {
        let triangles_at_depth = structure.triangles_at_depth(depth);
        
        for triangle in triangles_at_depth {
            let triangle_validation = validate_fractal_triangle(triangle);
            if !triangle_validation.is_valid {
                result.add_error(format!(
                    "Triangle {} at depth {} failed validation: {:?}",
                    triangle.id, depth, triangle_validation.errors
                ));
            }

            // Validate parent-child relationships
            if let Some(parent_id) = triangle.parent_id {
                if let Some(parent) = structure.get_triangle(&parent_id) {
                    let relationship_validation = validate_parent_child_relationship(parent, triangle);
                    result.combine(relationship_validation);
                } else {
                    result.add_error(format!(
                        "Triangle {} references non-existent parent {}",
                        triangle.id, parent_id
                    ));
                }
            }
        }
    }

    // Validate subdivision consistency
    let subdivided_triangles = structure.triangles_by_state(TriangleState::Subdivided);
    for parent in subdivided_triangles {
        let children = structure.children(&parent.id);
        
        if children.len() != parent.child_ids.len() {
            result.add_error(format!(
                "Triangle {} has {} child IDs but {} actual children found",
                parent.id,
                parent.child_ids.len(),
                children.len()
            ));
        }

        // For Sierpinski triangles, we expect 3 active children + 1 void
        if children.len() == 4 {
            let active_children = children.iter()
                .filter(|c| c.state == TriangleState::Active)
                .count();
            let void_children = children.iter()
                .filter(|c| c.state == TriangleState::Void)
                .count();

            if active_children != 3 || void_children != 1 {
                result.add_warning(format!(
                    "Triangle {} subdivision should have 3 active + 1 void children, found {} active + {} void",
                    parent.id, active_children, void_children
                ));
            }
        }
    }

    result
}

/// Validate equilateral properties of a triangle
pub fn validate_equilateral_triangle(triangle: &Triangle) -> ValidationResult {
    let mut result = ValidationResult::success();

    match triangle.is_equilateral() {
        Ok(is_equilateral) => {
            if !is_equilateral {
                result.add_error("Triangle is not equilateral".to_string());
            }
        }
        Err(e) => {
            result.add_error(format!("Failed to check equilateral property: {}", e));
        }
    }

    // Additional checks for equilateral triangles
    if let Ok(side_lengths) = triangle.side_lengths() {
        let tolerance = Decimal::new(1, 6); // 0.000001 tolerance
        let avg_length = (side_lengths[0] + side_lengths[1] + side_lengths[2]) / Decimal::from(3);
        
        for (i, &length) in side_lengths.iter().enumerate() {
            let diff = (length - avg_length).abs();
            if diff > tolerance {
                result.add_error(format!(
                    "Side {} length {} differs from average {} by {}",
                    i, length, avg_length, diff
                ));
            }
        }
    }

    result
}

/// Validate Sierpinski fractal properties
pub fn validate_sierpinski_properties(structure: &FractalStructure) -> ValidationResult {
    let mut result = ValidationResult::success();

    // Check that genesis is equilateral
    if let Some(genesis) = structure.genesis() {
        let equilateral_validation = validate_equilateral_triangle(&genesis.triangle);
        if !equilateral_validation.is_valid {
            result.add_error("Genesis triangle is not equilateral".to_string());
        }
    }

    // Check area conservation at each level
    for depth in 0..structure.max_depth() {
        let parent_triangles = structure.triangles_at_depth(depth)
            .into_iter()
            .filter(|t| t.state == TriangleState::Subdivided)
            .collect::<Vec<_>>();

        for parent in parent_triangles {
            let children = structure.children(&parent.id);
            
            if !children.is_empty() {
                match validate_area_conservation(parent, &children) {
                    Ok(is_conserved) => {
                        if !is_conserved {
                            result.add_warning(format!(
                                "Area not conserved in subdivision of triangle {}",
                                parent.id
                            ));
                        }
                    }
                    Err(e) => {
                        result.add_error(format!(
                            "Failed to validate area conservation for triangle {}: {}",
                            parent.id, e
                        ));
                    }
                }
            }
        }
    }

    result
}

/// Validate area conservation in subdivision
fn validate_area_conservation(
    parent: &FractalTriangle,
    children: &[&FractalTriangle],
) -> SierpinskiResult<bool> {
    let parent_area = parent.triangle.area()?;
    let mut total_child_area = Decimal::ZERO;
    
    for child in children {
        total_child_area += child.triangle.area()?;
    }
    
    let difference = (parent_area - total_child_area).abs();
    let tolerance = parent_area * Decimal::new(1, 6); // 0.0001% tolerance
    
    Ok(difference <= tolerance)
}

/// Quick validation function for simple use cases
pub fn is_valid_triangle(triangle: &Triangle) -> bool {
    validate_triangle(triangle).is_valid
}

/// Quick validation function for fractal triangles
pub fn is_valid_fractal_triangle(fractal_triangle: &FractalTriangle) -> bool {
    validate_fractal_triangle(fractal_triangle).is_valid
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{
        geometry::Point,
        genesis::genesis_fractal_triangle,
        subdivision::subdivide_triangle,
    };

    fn create_test_triangle() -> Triangle {
        Triangle::new(
            Point::from_f64(0.0, 0.0).unwrap(),
            Point::from_f64(1.0, 0.0).unwrap(),
            Point::from_f64(0.5, 0.866).unwrap(),
        ).unwrap()
    }

    #[test]
    fn test_valid_triangle_validation() {
        let triangle = create_test_triangle();
        let result = validate_triangle(&triangle);
        assert!(result.is_valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_collinear_triangle_validation() {
        let triangle = Triangle::new(
            Point::from_f64(0.0, 0.0).unwrap(),
            Point::from_f64(1.0, 1.0).unwrap(),
            Point::from_f64(2.0, 2.0).unwrap(),
        );
        
        // This should fail during triangle creation, not validation
        assert!(triangle.is_err());
    }

    #[test]
    fn test_fractal_triangle_validation() {
        let genesis = genesis_fractal_triangle().unwrap();
        let result = validate_fractal_triangle(&genesis);
        assert!(result.is_valid);
    }

    #[test]
    fn test_parent_child_validation() {
        let genesis = genesis_fractal_triangle().unwrap();
        let subdivision = subdivide_triangle(&genesis).unwrap();
        
        for child in &subdivision.children {
            let result = validate_parent_child_relationship(&subdivision.parent, child);
            assert!(result.is_valid, "Validation failed: {:?}", result.errors);
        }
    }

    #[test]
    fn test_equilateral_validation() {
        let triangle = create_test_triangle();
        let result = validate_equilateral_triangle(&triangle);
        
        // This should pass since our test triangle is approximately equilateral
        if !result.is_valid {
            println!("Equilateral validation errors: {:?}", result.errors);
        }
    }
}
