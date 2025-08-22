//! Sierpinski triangle subdivision algorithms

use rust_decimal::Decimal;
use uuid::Uuid;
use serde::{Deserialize, Serialize};

use crate::core::{
    triangle::Triangle,
    fractal::{FractalTriangle, FractalStructure},
    state::TriangleState,
    errors::{SierpinskiError, SierpinskiResult},
};

/// Result of a triangle subdivision operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubdivisionResult {
    /// The three child triangles created
    pub children: [FractalTriangle; 3],
    /// The central void triangle
    pub void_triangle: FractalTriangle,
    /// Original parent triangle (now in Subdivided state)
    pub parent: FractalTriangle,
}

/// Subdivide a triangle into the Sierpinski pattern
pub fn subdivide_triangle(
    parent: &FractalTriangle,
) -> SierpinskiResult<SubdivisionResult> {
    // Check if subdivision is allowed
    if !parent.can_subdivide() {
        return Err(SierpinskiError::subdivision(format!(
            "Triangle {} cannot be subdivided in state {}",
            parent.id, parent.state
        )));
    }

    // Get the midpoints of each side
    let midpoints = parent.triangle.side_midpoints();
    let [mid_ab, mid_bc, mid_ca] = midpoints;
    let [a, b, c] = parent.triangle.vertices();

    // Create the three child triangles
    let child_triangle_1 = Triangle::new(*a, mid_ab, mid_ca)?;
    let child_triangle_2 = Triangle::new(mid_ab, *b, mid_bc)?;
    let child_triangle_3 = Triangle::new(mid_ca, mid_bc, *c)?;

    // Create the central void triangle
    let void_triangle_geom = Triangle::new(mid_ab, mid_bc, mid_ca)?;

    // Create fractal triangles for children
    let child_1 = FractalTriangle::child(child_triangle_1, parent, 0)?;
    let child_2 = FractalTriangle::child(child_triangle_2, parent, 1)?;
    let child_3 = FractalTriangle::child(child_triangle_3, parent, 2)?;

    // Create void fractal triangle
    let mut void_triangle = FractalTriangle::child(void_triangle_geom, parent, 3)?;
    void_triangle.change_state(TriangleState::Void)?;

    // Create updated parent with new state
    let mut updated_parent = parent.clone();
    updated_parent.change_state(TriangleState::Subdivided)?;
    updated_parent.add_child(child_1.id);
    updated_parent.add_child(child_2.id);
    updated_parent.add_child(child_3.id);

    Ok(SubdivisionResult {
        children: [child_1, child_2, child_3],
        void_triangle,
        parent: updated_parent,
    })
}

/// Subdivide a triangle and add results to a fractal structure
pub fn subdivide_and_add_to_structure(
    structure: &mut FractalStructure,
    parent_id: &Uuid,
) -> SierpinskiResult<SubdivisionResult> {
    // Get the parent triangle
    let parent = structure
        .get_triangle(parent_id)
        .ok_or_else(|| SierpinskiError::subdivision("Parent triangle not found".to_string()))?
        .clone();

    // Perform subdivision
    let result = subdivide_triangle(&parent)?;

    // Update the structure with new triangles
    structure.add_triangle(result.parent.clone())?;
    for child in &result.children {
        structure.add_triangle(child.clone())?;
    }
    structure.add_triangle(result.void_triangle.clone())?;

    Ok(result)
}

/// Recursively subdivide to a specific depth
pub fn subdivide_to_depth(
    initial_triangle: FractalTriangle,
    target_depth: u8,
) -> SierpinskiResult<FractalStructure> {
    if target_depth > crate::MAX_SUBDIVISION_DEPTH {
        return Err(SierpinskiError::MaxDepthExceeded {
            max_depth: crate::MAX_SUBDIVISION_DEPTH,
        });
    }

    let mut structure = FractalStructure::new();
    structure.set_genesis(initial_triangle)?;

    let genesis_id = structure.genesis().unwrap().id;
    subdivide_recursive(&mut structure, genesis_id, target_depth)?;

    Ok(structure)
}

/// Recursive helper for subdivision
fn subdivide_recursive(
    structure: &mut FractalStructure,
    triangle_id: Uuid,
    target_depth: u8,
) -> SierpinskiResult<()> {
    let triangle = structure
        .get_triangle(&triangle_id)
        .ok_or_else(|| SierpinskiError::subdivision("Triangle not found".to_string()))?
        .clone();

    if triangle.depth >= target_depth {
        return Ok(());
    }

    if !triangle.can_subdivide() {
        return Ok(());
    }

    // Subdivide the triangle
    let result = subdivide_and_add_to_structure(structure, &triangle_id)?;

    // Recursively subdivide children
    for child in &result.children {
        subdivide_recursive(structure, child.id, target_depth)?;
    }

    Ok(())
}

/// Calculate the number of triangles at a given depth
pub fn triangles_at_depth(depth: u8) -> u64 {
    if depth == 0 {
        1 // Genesis triangle
    } else {
        3_u64.pow(depth as u32)
    }
}

/// Calculate total number of triangles up to a given depth
pub fn total_triangles_to_depth(depth: u8) -> u64 {
    let mut total = 0;
    for d in 0..=depth {
        total += triangles_at_depth(d);
    }
    total
}

/// Calculate the area ratio of child triangles to parent
pub fn child_area_ratio() -> Decimal {
    Decimal::new(1, 0) / Decimal::new(4, 0) // 1/4
}

/// Calculate the void area ratio to parent triangle
pub fn void_area_ratio() -> Decimal {
    Decimal::new(1, 0) / Decimal::new(4, 0) // 1/4
}

/// Validate a subdivision result
pub fn validate_subdivision(result: &SubdivisionResult) -> SierpinskiResult<bool> {
    // Check that parent is in subdivided state
    if result.parent.state != TriangleState::Subdivided {
        return Ok(false);
    }

    // Check that all children are active
    for child in &result.children {
        if child.state != TriangleState::Active {
            return Ok(false);
        }
        if child.parent_id != Some(result.parent.id) {
            return Ok(false);
        }
    }

    // Check that void is in void state
    if result.void_triangle.state != TriangleState::Void {
        return Ok(false);
    }

    // Verify area conservation (approximately)
    let parent_area = result.parent.area()?;
    let mut total_child_area = Decimal::ZERO;
    
    for child in &result.children {
        total_child_area += child.area()?;
    }
    total_child_area += result.void_triangle.area()?;

    let area_difference = (parent_area - total_child_area).abs();
    let tolerance = parent_area * Decimal::new(1, 6); // 0.0001% tolerance

    Ok(area_difference <= tolerance)
}

/// Get subdivision statistics for a fractal structure
#[derive(Debug, Clone)]
pub struct SubdivisionStats {
    pub total_triangles: usize,
    pub active_triangles: usize,
    pub subdivided_triangles: usize,
    pub void_triangles: usize,
    pub max_depth: u8,
    pub total_area: Decimal,
    pub active_area: Decimal,
}

impl SubdivisionStats {
    /// Calculate statistics for a fractal structure
    pub fn calculate(structure: &FractalStructure) -> SierpinskiResult<Self> {
        let active_triangles = structure.triangles_by_state(TriangleState::Active);
        let subdivided_triangles = structure.triangles_by_state(TriangleState::Subdivided);
        let void_triangles = structure.triangles_by_state(TriangleState::Void);
        let genesis_triangles = structure.triangles_by_state(TriangleState::Genesis);

        let mut total_area = Decimal::ZERO;
        let mut active_area = Decimal::ZERO;

        // Calculate total area from genesis and subdivided triangles
        for triangle in genesis_triangles.iter().chain(subdivided_triangles.iter()) {
            total_area += triangle.area()?;
        }

        // Calculate active area
        for triangle in &active_triangles {
            active_area += triangle.area()?;
        }

        Ok(SubdivisionStats {
            total_triangles: structure.total_triangles(),
            active_triangles: active_triangles.len(),
            subdivided_triangles: subdivided_triangles.len(),
            void_triangles: void_triangles.len(),
            max_depth: structure.max_depth(),
            total_area,
            active_area,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::genesis::genesis_fractal_triangle;

    #[test]
    fn test_subdivision() {
        let genesis = genesis_fractal_triangle().unwrap();
        let result = subdivide_triangle(&genesis).unwrap();

        assert_eq!(result.children.len(), 3);
        assert_eq!(result.parent.state, TriangleState::Subdivided);
        assert_eq!(result.void_triangle.state, TriangleState::Void);

        // Validate the subdivision
        assert!(validate_subdivision(&result).unwrap());
    }

    #[test]
    fn test_subdivision_to_depth() {
        let genesis = genesis_fractal_triangle().unwrap();
        let structure = subdivide_to_depth(genesis, 2).unwrap();

        assert_eq!(structure.max_depth(), 2);
        assert_eq!(structure.total_triangles(), total_triangles_to_depth(2) as usize);
    }

    #[test]
    fn test_triangles_at_depth_calculation() {
        assert_eq!(triangles_at_depth(0), 1);
        assert_eq!(triangles_at_depth(1), 3);
        assert_eq!(triangles_at_depth(2), 9);
        assert_eq!(triangles_at_depth(3), 27);
    }

    #[test]
    fn test_total_triangles_calculation() {
        assert_eq!(total_triangles_to_depth(0), 1);
        assert_eq!(total_triangles_to_depth(1), 4); // 1 + 3
        assert_eq!(total_triangles_to_depth(2), 13); // 1 + 3 + 9
    }

    #[test]
    fn test_subdivision_stats() {
        let genesis = genesis_fractal_triangle().unwrap();
        let structure = subdivide_to_depth(genesis, 1).unwrap();
        let stats = SubdivisionStats::calculate(&structure).unwrap();

        assert_eq!(stats.max_depth, 1);
        assert_eq!(stats.active_triangles, 3);
        assert_eq!(stats.subdivided_triangles, 1);
        assert_eq!(stats.void_triangles, 1);
    }
}
