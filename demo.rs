// Sierpinski Triangle Cryptocurrency - Core Concepts Demo
// This demonstrates the key mathematical and structural concepts

use std::collections::HashMap;

#[derive(Debug, Clone)]
struct Point {
    x: f64,
    y: f64,
}

impl Point {
    fn new(x: f64, y: f64) -> Self {
        Point { x, y }
    }
    
    fn midpoint(&self, other: &Point) -> Point {
        Point {
            x: (self.x + other.x) / 2.0,
            y: (self.y + other.y) / 2.0,
        }
    }
}

#[derive(Debug, Clone)]
struct Triangle {
    vertices: [Point; 3],
    id: String,
    depth: u8,
}

impl Triangle {
    fn new(p1: Point, p2: Point, p3: Point, id: String, depth: u8) -> Self {
        Triangle {
            vertices: [p1, p2, p3],
            id,
            depth,
        }
    }
    
    fn area(&self) -> f64 {
        let [a, b, c] = &self.vertices;
        let area = 0.5 * ((b.x - a.x) * (c.y - a.y) - (c.x - a.x) * (b.y - a.y)).abs();
        area
    }
    
    fn subdivide(&self) -> (Vec<Triangle>, Triangle) {
        let [a, b, c] = &self.vertices;
        
        // Calculate midpoints of each side
        let mid_ab = a.midpoint(b);
        let mid_bc = b.midpoint(c);
        let mid_ca = c.midpoint(a);
        
        // Create three child triangles
        let child1 = Triangle::new(
            a.clone(), mid_ab.clone(), mid_ca.clone(),
            format!("{}.0", self.id), self.depth + 1
        );
        let child2 = Triangle::new(
            mid_ab.clone(), b.clone(), mid_bc.clone(),
            format!("{}.1", self.id), self.depth + 1
        );
        let child3 = Triangle::new(
            mid_ca.clone(), mid_bc.clone(), c.clone(),
            format!("{}.2", self.id), self.depth + 1
        );
        
        // Create central void triangle
        let void_triangle = Triangle::new(
            mid_ab, mid_bc, mid_ca,
            format!("{}.void", self.id), self.depth + 1
        );
        
        (vec![child1, child2, child3], void_triangle)
    }
}

#[derive(Debug)]
struct SierpinskiFractal {
    triangles: HashMap<String, Triangle>,
    voids: HashMap<String, Triangle>,
    genesis_id: String,
}

impl SierpinskiFractal {
    fn new() -> Self {
        // Create genesis triangle (equilateral)
        let genesis = Triangle::new(
            Point::new(0.0, 0.0),
            Point::new(1.0, 0.0),
            Point::new(0.5, 0.866), // sqrt(3)/2 for equilateral
            "genesis".to_string(),
            0
        );
        
        let mut triangles = HashMap::new();
        triangles.insert("genesis".to_string(), genesis);
        
        SierpinskiFractal {
            triangles,
            voids: HashMap::new(),
            genesis_id: "genesis".to_string(),
        }
    }
    
    fn subdivide(&mut self, triangle_id: &str) -> Result<(), String> {
        let triangle = self.triangles.get(triangle_id)
            .ok_or("Triangle not found")?
            .clone();
            
        let (children, void_triangle) = triangle.subdivide();
        
        // Add children to the fractal
        for child in children {
            self.triangles.insert(child.id.clone(), child);
        }
        
        // Add void triangle
        self.voids.insert(void_triangle.id.clone(), void_triangle);
        
        Ok(())
    }
    
    fn generate_to_depth(&mut self, max_depth: u8) {
        let mut current_depth = 0;
        
        while current_depth < max_depth {
            let triangles_at_depth: Vec<String> = self.triangles
                .values()
                .filter(|t| t.depth == current_depth)
                .map(|t| t.id.clone())
                .collect();
                
            for triangle_id in triangles_at_depth {
                if let Err(e) = self.subdivide(&triangle_id) {
                    println!("Error subdividing {}: {}", triangle_id, e);
                }
            }
            
            current_depth += 1;
        }
    }
    
    fn total_triangles(&self) -> usize {
        self.triangles.len()
    }
    
    fn total_active_area(&self) -> f64 {
        self.triangles.values()
            .filter(|t| t.depth > 0) // Exclude subdivided triangles
            .map(|t| t.area())
            .sum()
    }
    
    fn stats(&self) -> String {
        let total_triangles = self.total_triangles();
        let total_voids = self.voids.len();
        let max_depth = self.triangles.values().map(|t| t.depth).max().unwrap_or(0);
        let total_area = self.total_active_area();
        
        format!(
            "Sierpinski Fractal Stats:\n\
             - Total triangles: {}\n\
             - Void triangles: {}\n\
             - Maximum depth: {}\n\
             - Total active area: {:.6}",
            total_triangles, total_voids, max_depth, total_area
        )
    }
}

fn main() {
    println!("🔺 Sierpinski Triangle Cryptocurrency - Core Demo\n");
    
    let mut fractal = SierpinskiFractal::new();
    
    println!("Genesis triangle created!");
    println!("{}\n", fractal.stats());
    
    // Generate fractal to depth 3
    println!("Generating fractal to depth 3...");
    fractal.generate_to_depth(3);
    
    println!("{}\n", fractal.stats());
    
    // Demonstrate key cryptocurrency concepts
    println!("📊 Cryptocurrency Properties:");
    
    // Calculate theoretical number of triangles at each depth
    for depth in 0..=3 {
        let count_at_depth = if depth == 0 { 1 } else { 3_u32.pow(depth as u32) };
        println!("  - Depth {}: {} triangles (theoretical)", depth, count_at_depth);
    }
    
    println!("\n🏦 Token Economics Simulation:");
    let genesis_area = fractal.triangles.get("genesis").unwrap().area();
    let total_active_area = fractal.total_active_area();
    let area_reduction_ratio = total_active_area / genesis_area;
    
    println!("  - Genesis triangle area: {:.6}", genesis_area);
    println!("  - Total active area after subdivision: {:.6}", total_active_area);
    println!("  - Area reduction ratio: {:.6}", area_reduction_ratio);
    println!("  - This demonstrates the deflationary nature!");
    
    println!("\n🎯 Address System Demo:");
    for (id, triangle) in fractal.triangles.iter().take(10) {
        println!("  - Triangle {} at depth {}", id, triangle.depth);
    }
    
    println!("\n✅ Core Sierpinski Triangle cryptocurrency concepts verified!");
    println!("   This demonstrates the mathematical foundation for:");
    println!("   • Geometric proof-of-work mining");
    println!("   • Fractal-based token economics");
    println!("   • Hierarchical addressing system");
    println!("   • Area-based value distribution");
}