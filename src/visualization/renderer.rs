//! SVG rendering for Sierpinski triangle fractals

use rust_decimal::Decimal;
use std::fmt::Write;

use crate::core::{
    fractal::FractalStructure,
    geometry::Point,
    state::TriangleState,
    errors::SierpinskiResult,
};

/// Rendering options for SVG output
#[derive(Debug, Clone)]
pub struct RenderOptions {
    pub width: u32,
    pub height: u32,
    pub show_addresses: bool,
    pub show_void_triangles: bool,
    pub stroke_width: f64,
    pub colors: ColorScheme,
}

impl Default for RenderOptions {
    fn default() -> Self {
        RenderOptions {
            width: 800,
            height: 800,
            show_addresses: false,
            show_void_triangles: true,
            stroke_width: 1.0,
            colors: ColorScheme::default(),
        }
    }
}

/// Color scheme for rendering
#[derive(Debug, Clone)]
pub struct ColorScheme {
    pub genesis: String,
    pub active: String,
    pub subdivided: String,
    pub void_triangle: String,
    pub stroke: String,
    pub text: String,
    pub background: String,
}

impl Default for ColorScheme {
    fn default() -> Self {
        ColorScheme {
            genesis: "#FF6B6B".to_string(),      // Red
            active: "#4ECDC4".to_string(),       // Teal
            subdivided: "#45B7D1".to_string(),   // Blue
            void_triangle: "#F9F9F9".to_string(), // Light gray
            stroke: "#2C3E50".to_string(),       // Dark blue-gray
            text: "#2C3E50".to_string(),         // Dark blue-gray
            background: "#FFFFFF".to_string(),   // White
        }
    }
}

/// Render a fractal structure to SVG
pub fn render_fractal_svg(
    structure: &FractalStructure,
    width: u32,
    height: u32,
    show_addresses: bool,
) -> SierpinskiResult<String> {
    let options = RenderOptions {
        width,
        height,
        show_addresses,
        ..Default::default()
    };
    
    render_fractal_svg_with_options(structure, &options)
}

/// Render a fractal structure to SVG with custom options
pub fn render_fractal_svg_with_options(
    structure: &FractalStructure,
    options: &RenderOptions,
) -> SierpinskiResult<String> {
    let mut svg = String::new();
    
    // Calculate bounds
    let bounds = calculate_bounds(structure)?;
    let scale = calculate_scale(&bounds, options.width, options.height);
    
    // SVG header
    writeln!(
        &mut svg,
        r#"<svg width="{}" height="{}" xmlns="http://www.w3.org/2000/svg">"#,
        options.width, options.height
    ).unwrap();
    
    // Background
    writeln!(
        &mut svg,
        r#"<rect width="100%" height="100%" fill="{}"/>"#,
        options.colors.background
    ).unwrap();
    
    // Define styles
    write_styles(&mut svg, options)?;
    
    // Render triangles by depth (background to foreground)
    for depth in (0..=structure.max_depth()).rev() {
        let triangles = structure.triangles_at_depth(depth);
        
        for triangle in triangles {
            render_triangle(&mut svg, triangle, &bounds, scale, options)?;
        }
    }
    
    // Render addresses if requested
    if options.show_addresses {
        render_addresses(&mut svg, structure, &bounds, scale, options)?;
    }
    
    // SVG footer
    writeln!(&mut svg, "</svg>").unwrap();
    
    Ok(svg)
}

/// Calculate the bounding box of all triangles
fn calculate_bounds(structure: &FractalStructure) -> SierpinskiResult<Bounds> {
    let mut min_x = Decimal::MAX;
    let mut max_x = Decimal::MIN;
    let mut min_y = Decimal::MAX;
    let mut max_y = Decimal::MIN;
    
    for depth in 0..=structure.max_depth() {
        let triangles = structure.triangles_at_depth(depth);
        
        for triangle in triangles {
            for vertex in triangle.triangle.vertices() {
                if vertex.x < min_x { min_x = vertex.x; }
                if vertex.x > max_x { max_x = vertex.x; }
                if vertex.y < min_y { min_y = vertex.y; }
                if vertex.y > max_y { max_y = vertex.y; }
            }
        }
    }
    
    // Add padding
    let padding = (max_x - min_x) * Decimal::new(1, 1); // 10% padding
    
    Ok(Bounds {
        min_x: min_x - padding,
        max_x: max_x + padding,
        min_y: min_y - padding,
        max_y: max_y + padding,
    })
}

/// Calculate scale factor for coordinate transformation
fn calculate_scale(bounds: &Bounds, width: u32, height: u32) -> Scale {
    let bounds_width = bounds.max_x - bounds.min_x;
    let bounds_height = bounds.max_y - bounds.min_y;
    
    let scale_x = Decimal::try_from(width as f64).unwrap() / bounds_width;
    let scale_y = Decimal::try_from(height as f64).unwrap() / bounds_height;
    
    // Use the smaller scale to maintain aspect ratio
    let scale = if scale_x < scale_y { scale_x } else { scale_y };
    
    Scale {
        factor: scale,
        offset_x: bounds.min_x,
        offset_y: bounds.min_y,
        canvas_width: width,
        canvas_height: height,
    }
}

/// Transform a point from world coordinates to SVG coordinates
fn transform_point(point: &Point, _bounds: &Bounds, scale: &Scale) -> (f64, f64) {
    let x = ((point.x - scale.offset_x) * scale.factor).to_string().parse::<f64>().unwrap_or(0.0);
    let y = (scale.canvas_height as f64) - ((point.y - scale.offset_y) * scale.factor).to_string().parse::<f64>().unwrap_or(0.0);
    (x, y)
}

/// Render a single triangle
fn render_triangle(
    svg: &mut String,
    triangle: &crate::core::fractal::FractalTriangle,
    bounds: &Bounds,
    scale: Scale,
    options: &RenderOptions,
) -> SierpinskiResult<()> {
    let vertices = triangle.triangle.vertices();
    let (x1, y1) = transform_point(&vertices[0], bounds, &scale);
    let (x2, y2) = transform_point(&vertices[1], bounds, &scale);
    let (x3, y3) = transform_point(&vertices[2], bounds, &scale);
    
    let fill_color = match triangle.state {
        TriangleState::Genesis => &options.colors.genesis,
        TriangleState::Active => &options.colors.active,
        TriangleState::Subdivided => &options.colors.subdivided,
        TriangleState::Void => {
            if !options.show_void_triangles {
                return Ok(());
            }
            &options.colors.void_triangle
        }
        TriangleState::Inactive => &options.colors.active,
    };
    
    writeln!(
        svg,
        r#"<polygon points="{:.2},{:.2} {:.2},{:.2} {:.2},{:.2}" fill="{}" stroke="{}" stroke-width="{}" opacity="{}"/>"#,
        x1, y1, x2, y2, x3, y3,
        fill_color,
        options.colors.stroke,
        options.stroke_width,
        if triangle.state == TriangleState::Void { 0.3 } else { 0.8 }
    ).unwrap();
    
    Ok(())
}

/// Render triangle addresses
fn render_addresses(
    svg: &mut String,
    structure: &FractalStructure,
    bounds: &Bounds,
    scale: Scale,
    options: &RenderOptions,
) -> SierpinskiResult<()> {
    for depth in 0..=structure.max_depth() {
        let triangles = structure.triangles_at_depth(depth);
        
        for triangle in triangles {
            // Skip void triangles for address rendering
            if triangle.state == TriangleState::Void {
                continue;
            }
            
            let centroid = triangle.triangle.centroid();
            let (x, y) = transform_point(&centroid, bounds, &scale);
            
            let font_size = (12.0 - (depth as f64 * 1.5)).max(6.0);
            
            writeln!(
                svg,
                r#"<text x="{:.2}" y="{:.2}" font-family="monospace" font-size="{}" fill="{}" text-anchor="middle" dominant-baseline="middle">{}</text>"#,
                x, y, font_size, options.colors.text, triangle.address
            ).unwrap();
        }
    }
    
    Ok(())
}

/// Write CSS styles to SVG
fn write_styles(svg: &mut String, options: &RenderOptions) -> SierpinskiResult<()> {
    writeln!(svg, "<defs>").unwrap();
    writeln!(svg, "<style>").unwrap();
    writeln!(svg, ".triangle-genesis {{ fill: {}; }}", options.colors.genesis).unwrap();
    writeln!(svg, ".triangle-active {{ fill: {}; }}", options.colors.active).unwrap();
    writeln!(svg, ".triangle-subdivided {{ fill: {}; }}", options.colors.subdivided).unwrap();
    writeln!(svg, ".triangle-void {{ fill: {}; opacity: 0.3; }}", options.colors.void_triangle).unwrap();
    writeln!(svg, ".triangle-stroke {{ stroke: {}; stroke-width: {}; }}", options.colors.stroke, options.stroke_width).unwrap();
    writeln!(svg, "</style>").unwrap();
    writeln!(svg, "</defs>").unwrap();
    Ok(())
}

/// Coordinate bounds
#[derive(Debug, Clone)]
struct Bounds {
    min_x: Decimal,
    max_x: Decimal,
    min_y: Decimal,
    max_y: Decimal,
}

/// Scaling information
#[derive(Debug, Clone, Copy)]
struct Scale {
    factor: Decimal,
    offset_x: Decimal,
    offset_y: Decimal,
    canvas_width: u32,
    canvas_height: u32,
}

/// Generate a simple fractal visualization for testing
pub fn generate_test_svg() -> String {
    use crate::core::{
        genesis::genesis_fractal_triangle,
        subdivision::subdivide_to_depth,
    };
    
    let genesis = genesis_fractal_triangle().unwrap();
    let structure = subdivide_to_depth(genesis, 3).unwrap();
    
    render_fractal_svg(&structure, 800, 800, true).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{
        genesis::genesis_fractal_triangle,
        subdivision::subdivide_to_depth,
    };

    #[test]
    fn test_svg_generation() {
        let genesis = genesis_fractal_triangle().unwrap();
        let structure = subdivide_to_depth(genesis, 2).unwrap();
        
        let svg = render_fractal_svg(&structure, 400, 400, false).unwrap();
        
        assert!(svg.contains("<svg"));
        assert!(svg.contains("</svg>"));
        assert!(svg.contains("polygon"));
    }

    #[test]
    fn test_bounds_calculation() {
        let genesis = genesis_fractal_triangle().unwrap();
        let structure = subdivide_to_depth(genesis, 1).unwrap();
        
        let bounds = calculate_bounds(&structure).unwrap();
        
        assert!(bounds.max_x > bounds.min_x);
        assert!(bounds.max_y > bounds.min_y);
    }

    #[test]
    fn test_svg_with_addresses() {
        let genesis = genesis_fractal_triangle().unwrap();
        let structure = subdivide_to_depth(genesis, 1).unwrap();
        
        let svg = render_fractal_svg(&structure, 400, 400, true).unwrap();
        
        assert!(svg.contains("<text"));
        assert!(svg.contains("genesis"));
    }
}
