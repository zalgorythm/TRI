//! Command-line interface for TriadChain operations

use clap::{Args, Parser, Subcommand};
use rust_decimal::Decimal;
use serde_json;
use std::{fs, path::PathBuf};

use triadchain::{
    core::{
        geometry::Point,
        genesis::{genesis_fractal_triangle, genesis_triangle_bounded},
        subdivision::{subdivide_to_depth, SubdivisionStats},
        validation::{validate_fractal_structure, validate_sierpinski_properties},
        fractal::FractalStructure,
        address::TriangleAddress,
    },
    visualization::renderer::render_fractal_svg,
};

#[derive(Parser)]
#[command(name = "triadchain_cli")]
#[command(about = "A CLI for TriadChain geometric cryptocurrency operations")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a TriadChain triangle fractal
    Generate(GenerateArgs),
    /// Validate a fractal structure
    Validate(ValidateArgs),
    /// Display information about a fractal
    Info(InfoArgs),
    /// Render a fractal to SVG
    Render(RenderArgs),
    /// Address operations
    Address(AddressArgs),
}

#[derive(Args)]
struct GenerateArgs {
    /// Maximum subdivision depth
    #[arg(short, long, default_value = "3")]
    depth: u8,
    
    /// Output file path
    #[arg(short, long)]
    output: Option<PathBuf>,
    
    /// Genesis triangle bounds (min_x,min_y,max_x,max_y)
    #[arg(long)]
    bounds: Option<String>,
    
    /// Pretty print JSON output
    #[arg(long)]
    pretty: bool,
}

#[derive(Args)]
struct ValidateArgs {
    /// Input fractal file
    #[arg(short, long)]
    input: PathBuf,
    
    /// Validate TriadChain-specific properties
    #[arg(long)]
    sierpinski: bool,
    
    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Args)]
struct InfoArgs {
    /// Input fractal file
    #[arg(short, long)]
    input: PathBuf,
    
    /// Show detailed statistics
    #[arg(short, long)]
    stats: bool,
}

#[derive(Args)]
struct RenderArgs {
    /// Input fractal file
    #[arg(short, long)]
    input: PathBuf,
    
    /// Output SVG file
    #[arg(short, long)]
    output: PathBuf,
    
    /// Image width
    #[arg(long, default_value = "800")]
    width: u32,
    
    /// Image height
    #[arg(long, default_value = "800")]
    height: u32,
    
    /// Show triangle addresses
    #[arg(long)]
    show_addresses: bool,
}

#[derive(Args)]
struct AddressArgs {
    #[command(subcommand)]
    operation: AddressOperation,
}

#[derive(Subcommand)]
enum AddressOperation {
    /// Parse an address from string
    Parse {
        /// Address string (e.g., "0.1.2" or "genesis")
        address: String,
    },
    /// Generate children of an address
    Children {
        /// Parent address
        address: String,
    },
    /// Find parent of an address
    Parent {
        /// Child address
        address: String,
    },
    /// Check if two addresses are related
    Related {
        /// First address
        address1: String,
        /// Second address
        address2: String,
    },
}

fn main() {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Generate(args) => handle_generate(args),
        Commands::Validate(args) => handle_validate(args),
        Commands::Info(args) => handle_info(args),
        Commands::Render(args) => handle_render(args),
        Commands::Address(args) => handle_address(args),
    }
}

fn handle_generate(args: GenerateArgs) {
    println!("Generating TriadChain triangle to depth {}...", args.depth);
    
    // Create genesis triangle
    let genesis = if let Some(bounds_str) = args.bounds {
        let bounds: Vec<f64> = bounds_str
            .split(',')
            .map(|s| s.parse().expect("Invalid bounds format"))
            .collect();
        
        if bounds.len() != 4 {
            eprintln!("Bounds must be in format: min_x,min_y,max_x,max_y");
            std::process::exit(1);
        }
        
        let triangle = genesis_triangle_bounded(
            Decimal::try_from(bounds[0]).unwrap(),
            Decimal::try_from(bounds[2]).unwrap(),
            Decimal::try_from(bounds[1]).unwrap(),
            Decimal::try_from(bounds[3]).unwrap(),
        ).expect("Failed to create bounded genesis triangle");
        
        triadchain::FractalTriangle::genesis(triangle)
    } else {
        genesis_fractal_triangle().expect("Failed to create genesis triangle")
    };
    
    // Generate fractal structure
    let structure = subdivide_to_depth(genesis, args.depth)
        .expect("Failed to generate fractal structure");
    
    println!("Generated {} triangles", structure.total_triangles());
    
    // Serialize and save
    let json = if args.pretty {
        serde_json::to_string_pretty(&structure)
    } else {
        serde_json::to_string(&structure)
    }.expect("Failed to serialize structure");
    
    if let Some(output_path) = args.output {
        fs::write(&output_path, json)
            .expect("Failed to write output file");
        println!("Saved to: {}", output_path.display());
    } else {
        println!("{}", json);
    }
}

fn handle_validate(args: ValidateArgs) {
    println!("Validating fractal structure...");
    
    let json = fs::read_to_string(&args.input)
        .expect("Failed to read input file");
    
    let structure: FractalStructure = serde_json::from_str(&json)
        .expect("Failed to parse fractal structure");
    
    let validation_result = validate_fractal_structure(&structure);
    
    if validation_result.is_valid {
        println!("✓ Fractal structure is valid");
    } else {
        println!("✗ Fractal structure validation failed:");
        for error in &validation_result.errors {
            println!("  ERROR: {}", error);
        }
    }
    
    if !validation_result.warnings.is_empty() && args.verbose {
        println!("\nWarnings:");
        for warning in &validation_result.warnings {
            println!("  WARNING: {}", warning);
        }
    }
    
    if args.sierpinski {
        println!("\nValidating TriadChain-specific properties...");
        let triadchain_result = validate_sierpinski_properties(&structure);
        
        if triadchain_result.is_valid {
            println!("✓ TriadChain properties are valid");
        } else {
            println!("✗ TriadChain validation failed:");
            for error in &triadchain_result.errors {
                println!("  ERROR: {}", error);
            }
        }
    }
}

fn handle_info(args: InfoArgs) {
    let json = fs::read_to_string(&args.input)
        .expect("Failed to read input file");
    
    let structure: FractalStructure = serde_json::from_str(&json)
        .expect("Failed to parse fractal structure");
    
    println!("Fractal Structure Information");
    println!("============================");
    println!("Total triangles: {}", structure.total_triangles());
    println!("Maximum depth: {}", structure.max_depth());
    
    if let Some(genesis) = structure.genesis() {
        println!("Genesis triangle ID: {}", genesis.id);
        if let Ok(area) = genesis.triangle.area() {
            println!("Genesis area: {}", area);
        }
    }
    
    // Show triangles by depth
    for depth in 0..=structure.max_depth() {
        let triangles_at_depth = structure.triangles_at_depth(depth);
        println!("Depth {}: {} triangles", depth, triangles_at_depth.len());
    }
    
    if args.stats {
        println!("\nDetailed Statistics");
        println!("==================");
        
        if let Ok(stats) = SubdivisionStats::calculate(&structure) {
            println!("Active triangles: {}", stats.active_triangles);
            println!("Subdivided triangles: {}", stats.subdivided_triangles);
            println!("Void triangles: {}", stats.void_triangles);
            println!("Total area: {}", stats.total_area);
            println!("Active area: {}", stats.active_area);
        }
    }
}

fn handle_render(args: RenderArgs) {
    println!("Rendering fractal to SVG...");
    
    let json = fs::read_to_string(&args.input)
        .expect("Failed to read input file");
    
    let structure: FractalStructure = serde_json::from_str(&json)
        .expect("Failed to parse fractal structure");
    
    let svg = render_fractal_svg(&structure, args.width, args.height, args.show_addresses)
        .expect("Failed to render SVG");
    
    fs::write(&args.output, svg)
        .expect("Failed to write SVG file");
    
    println!("Rendered to: {}", args.output.display());
}

fn handle_address(args: AddressArgs) {
    match args.operation {
        AddressOperation::Parse { address } => {
            match TriangleAddress::from_string_representation(&address) {
                Ok(addr) => {
                    println!("Address: {}", addr);
                    println!("Depth: {}", addr.depth());
                    println!("Is genesis: {}", addr.is_genesis());
                    println!("Is void: {}", addr.is_void());
                    if let Some(component) = addr.last_component() {
                        println!("Last component: {}", component);
                    }
                    println!("Components: {:?}", addr.components());
                }
                Err(e) => {
                    eprintln!("Error parsing address: {}", e);
                    std::process::exit(1);
                }
            }
        }
        AddressOperation::Children { address } => {
            match TriangleAddress::from_string_representation(&address) {
                Ok(addr) => {
                    let children = addr.children();
                    println!("Children of {}:", addr);
                    for child in children {
                        println!("  {}", child);
                    }
                }
                Err(e) => {
                    eprintln!("Error parsing address: {}", e);
                    std::process::exit(1);
                }
            }
        }
        AddressOperation::Parent { address } => {
            match TriangleAddress::from_string_representation(&address) {
                Ok(addr) => {
                    if let Some(parent) = addr.parent() {
                        println!("Parent of {}: {}", addr, parent);
                    } else {
                        println!("{} has no parent (it's the genesis)", addr);
                    }
                }
                Err(e) => {
                    eprintln!("Error parsing address: {}", e);
                    std::process::exit(1);
                }
            }
        }
        AddressOperation::Related { address1, address2 } => {
            match (
                TriangleAddress::from_string_representation(&address1),
                TriangleAddress::from_string_representation(&address2),
            ) {
                (Ok(addr1), Ok(addr2)) => {
                    println!("Analyzing relationship between {} and {}:", addr1, addr2);
                    
                    if addr1.is_child_of(&addr2) {
                        println!("{} is a child of {}", addr1, addr2);
                    } else if addr2.is_child_of(&addr1) {
                        println!("{} is a child of {}", addr2, addr1);
                    } else if addr1.is_ancestor_of(&addr2) {
                        println!("{} is an ancestor of {}", addr1, addr2);
                    } else if addr2.is_ancestor_of(&addr1) {
                        println!("{} is an ancestor of {}", addr2, addr1);
                    } else {
                        let common = addr1.common_ancestor(&addr2);
                        println!("No direct parent-child relationship");
                        println!("Common ancestor: {}", common);
                    }
                    
                    let siblings1 = addr1.siblings();
                    if siblings1.contains(&addr2) {
                        println!("They are siblings");
                    }
                }
                (Err(e), _) => {
                    eprintln!("Error parsing first address: {}", e);
                    std::process::exit(1);
                }
                (_, Err(e)) => {
                    eprintln!("Error parsing second address: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }
}
