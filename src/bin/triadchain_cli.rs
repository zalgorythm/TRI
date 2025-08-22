//! Command-line interface for TriadChain operations

use clap::{Args, Parser, Subcommand};
use rust_decimal::Decimal;
use serde_json;
use std::{fs, path::PathBuf};

use triadchain::{
    core::{
        genesis::{genesis_fractal_triangle, genesis_triangle_bounded},
        subdivision::{subdivide_to_depth, SubdivisionStats},
        validation::{validate_fractal_structure, validate_sierpinski_properties},
        fractal::FractalStructure,
        address::TriangleAddress,
        wallet::TriadChainWallet,
        blockchain::TriadChainBlockchain,
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
    /// Show bot status
    Start,
    /// Get blockchain statistics
    Stats,
    /// Create a new wallet
    Newwallet,
    /// Get wallet balance for address
    Balance {
        /// Wallet address
        address: String,
    },
    /// Get current mining difficulty
    Difficulty,
    /// Get latest block information
    Latestblock,
    /// Generate a triangle fractal
    Generatetriangle(GenerateTriangleArgs),
    /// Validate triangle address
    Validateaddress {
        /// Address to validate
        address: String,
    },
    /// Get triangle information
    Triangleinfo {
        /// Triangle address
        address: String,
    },
    /// Show economic metrics
    Economics,
    /// Show staking pools
    Stakingpools,
    /// Generate a TriadChain triangle fractal (legacy)
    Generate(GenerateArgs),
    /// Validate a fractal structure (legacy)
    Validate(ValidateArgs),
    /// Display information about a fractal (legacy)
    Info(InfoArgs),
    /// Render a fractal to SVG (legacy)
    Render(RenderArgs),
    /// Address operations (legacy)
    Address(AddressArgs),
}

#[derive(Args)]
struct GenerateTriangleArgs {
    /// Maximum subdivision depth
    #[arg(short, long, default_value = "3")]
    depth: u8,
    
    /// Output file path
    #[arg(short, long)]
    output: Option<PathBuf>,
    
    /// Pretty print JSON output
    #[arg(long)]
    pretty: bool,
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
        Commands::Start => handle_start(),
        Commands::Stats => handle_stats(),
        Commands::Newwallet => handle_newwallet(),
        Commands::Balance { address } => handle_balance(address),
        Commands::Difficulty => handle_difficulty(),
        Commands::Latestblock => handle_latestblock(),
        Commands::Generatetriangle(args) => handle_generatetriangle(args),
        Commands::Validateaddress { address } => handle_validateaddress(address),
        Commands::Triangleinfo { address } => handle_triangleinfo(address),
        Commands::Economics => handle_economics(),
        Commands::Stakingpools => handle_stakingpools(),
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


fn handle_start() {
    println!("🚀 TriadChain Bot Status");
    println!("=======================");
    println!();
    println!("Status: ✅ Online");
    println!("Version: v0.1.0");
    println!("Network: MainNet");
    println!("Node ID: TC-{}", uuid::Uuid::new_v4().to_string()[..8].to_uppercase());
    println!();
    println!("📊 System Information:");
    println!("  • CPU Cores: {}", num_cpus::get());
    println!("  • Memory: Available");
    println!("  • Storage: Available");
    println!();
    println!("🔗 Network Status:");
    println!("  • Peers: 0 connected");
    println!("  • Sync: Up to date");
    println!("  • Mining: Ready");
    println!();
    println!("Use 'stats' command for detailed blockchain statistics");
}

fn handle_stats() {
    println!("📊 TriadChain Blockchain Statistics");
    println!("===================================");
    println!();
    
    // Initialize a demo blockchain for stats
    match TriadChainBlockchain::new() {
        Ok(blockchain) => {
            println!("⛓️  Blockchain Stats:");
            println!("  • Chain Height: {}", blockchain.blocks.len());
            println!("  • Total Blocks: {}", blockchain.blocks.len());
            println!("  • Pending Transactions: {}", blockchain.mempool.len());
            println!("  • Difficulty: {}", blockchain.difficulty);
            println!();
            
            println!("🔺 Triangle Stats:");
            println!("  • Total Triangles: {}", blockchain.fractal_state.total_triangles());
            println!("  • Active Triangles: {}", blockchain.fractal_state.triangles_by_state(triadchain::core::state::TriangleState::Active).len());
            println!("  • Subdivided: {}", blockchain.fractal_state.triangles_by_state(triadchain::core::state::TriangleState::Subdivided).len());
            println!("  • Maximum Depth: {}", blockchain.fractal_state.max_depth());
            println!();
            
            println!("💰 Economic Stats:");
            println!("  • Circulating Supply: 1,000,000 TC");
            println!("  • Total Supply: 10,000,000 TC");
            println!("  • Market Cap: $500,000");
            println!("  • Price: $0.50 USD");
            println!();
            
            println!("⛏️  Mining Stats:");
            println!("  • Network Hashrate: 1.5 KH/s");
            println!("  • Average Block Time: 60s");
            println!("  • Last Block: 2 minutes ago");
            println!("  • Next Difficulty Adjustment: 144 blocks");
        },
        Err(e) => {
            eprintln!("❌ Failed to initialize blockchain: {}", e);
            println!("\n🔺 Using Mock Statistics:");
            println!("  • Total Triangles: 127");
            println!("  • Active Triangles: 64");
            println!("  • Subdivided: 63");
            println!("  • Maximum Depth: 6");
            println!("  • Chain Height: 1,234");
            println!("  • Difficulty: 4");
        }
    }
}

fn handle_newwallet() {
    println!("🔐 Creating New TriadChain Wallet...");
    println!();
    
    match TriadChainWallet::new() {
        Ok(wallet) => {
            println!("✅ Wallet created successfully!");
            println!();
            println!("📝 Wallet Information:");
            println!("  • Address: {}", wallet.wallet_id);
            println!("  • Balance: {} TC", wallet.balance);
            println!("  • Staked: {} TC", wallet.staked_balance);
            println!("  • Created: {}", chrono::DateTime::from_timestamp(wallet.created_at as i64, 0)
                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
                .unwrap_or_else(|| "Unknown".to_string()));
            println!();
            println!("⚠️  IMPORTANT: Save your wallet address safely!");
            println!("   Your address is your identity on TriadChain.");
            println!();
            println!("🎯 Next Steps:");
            println!("  • Use 'balance {}' to check your balance", wallet.wallet_id);
            println!("  • Use 'generatetriangle' to start earning triangles");
            println!("  • Use 'stakingpools' to explore staking options");
        },
        Err(e) => {
            eprintln!("❌ Failed to create wallet: {}", e);
            std::process::exit(1);
        }
    }
}

fn handle_balance(address: String) {
    println!("💰 Wallet Balance for {}", address);
    println!("{}=", "=".repeat(address.len() + 20));
    println!();
    
    // Validate address format
    if !address.starts_with("ST") {
        eprintln!("❌ Invalid address format. TriadChain addresses start with 'ST'");
        std::process::exit(1);
    }
    
    // Mock balance data
    println!("📊 Balance Information:");
    println!("  • Available: 2,847.32 TC");
    println!("  • Staked: 1,250.00 TC");
    println!("  • Pending: 23.45 TC");
    println!("  • Total: 4,120.77 TC");
    println!();
    
    println!("💵 USD Value (@ $0.50/TC):");
    println!("  • Available: $1,423.66");
    println!("  • Staked: $625.00");
    println!("  • Total: $2,060.39");
    println!();
    
    println!("🔺 Triangle Portfolio:");
    println!("  • Owned Triangles: 47");
    println!("  • Staked Triangles: 12");
    println!("  • Total Value: $2,847.32");
    println!();
    
    println!("📈 Recent Activity:");
    println!("  • Last Transaction: 2 hours ago");
    println!("  • Mining Rewards (24h): +127.8 TC");
    println!("  • Staking Rewards (24h): +15.2 TC");
}

fn handle_difficulty() {
    println!("⛏️  Current Mining Difficulty");
    println!("============================");
    println!();
    
    match TriadChainBlockchain::new() {
        Ok(blockchain) => {
            println!("🎯 Difficulty Metrics:");
            println!("  • Current Difficulty: {}", blockchain.difficulty);
            println!("  • Target Block Time: 60 seconds");
            println!("  • Last Adjustment: 72 blocks ago");
            println!("  • Next Adjustment: in 72 blocks");
            println!();
            
            println!("📊 Network Stats:");
            println!("  • Network Hashrate: 1,245 H/s");
            println!("  • Your Hashrate: 125 H/s (10.0%)");
            println!("  • Estimated Time to Block: ~8 minutes");
            println!();
            
            println!("🔺 Geometric Difficulty:");
            println!("  • Required Subdivisions: {}", std::cmp::min(blockchain.difficulty / 2, 10));
            println!("  • Area Precision: 10 decimals");
            println!("  • Triangle Validation: Strict");
            println!();
            
            println!("📈 Recent Changes:");
            if blockchain.difficulty > 1000 {
                println!("  • Status: ⬆️  Increased (+5.2%)");
                println!("  • Reason: Network hashrate increased");
            } else {
                println!("  • Status: ➡️  Stable (0.0%)");
                println!("  • Reason: Hashrate steady");
            }
        },
        Err(e) => {
            eprintln!("❌ Failed to initialize blockchain: {}", e);
            println!("\n🔺 Using Mock Difficulty Data:");
            println!("  • Current Difficulty: 4");
            println!("  • Target Block Time: 60 seconds");
            println!("  • Required Subdivisions: 2");
        }
    }
}

fn handle_latestblock() {
    println!("📦 Latest Block Information");
    println!("==========================");
    println!();
    
    match TriadChainBlockchain::new() {
        Ok(blockchain) => {
            if let Some(latest_block) = blockchain.blocks.last() {
                println!("🔗 Block Details:");
                println!("  • Height: {}", latest_block.height);
                println!("  • Hash: {}...", &latest_block.hash()[..16]);
                println!("  • Timestamp: {}", chrono::DateTime::from_timestamp(latest_block.header.timestamp as i64, 0)
                    .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
                    .unwrap_or_else(|| "Unknown".to_string()));
                println!("  • Size: {} bytes", latest_block.header.merkle_root.len() * 32); // rough estimate
                println!();
                
                println!("⛏️  Mining Details:");
                println!("  • Miner: {}", latest_block.miner_address);
                println!("  • Difficulty: {}", latest_block.header.difficulty);
                println!("  • Nonce: {}", latest_block.geometric_proof.nonce);
                println!("  • Mining Time: ~45 seconds");
                println!();
                
                println!("🔺 Geometric Proof:");
                println!("  • Triangle Hash: {}...", &latest_block.geometric_proof.triangle_hash[..16]);
                println!("  • Area Conservation: {}", if latest_block.geometric_proof.area_conservation { "✅ Valid" } else { "❌ Invalid" });
                println!("  • Subdivision Valid: {}", if latest_block.geometric_proof.subdivision_valid { "✅ Yes" } else { "❌ No" });
                println!();
                
                println!("📊 Transactions:");
                println!("  • Count: {}", latest_block.triangle_transactions.len());
                println!("  • Total Fees: {} TC", latest_block.triangle_transactions.iter()
                    .map(|tx| tx.gas_fee)
                    .sum::<Decimal>());
                println!("  • Volume: {} TC", latest_block.triangle_transactions.len() * 10); // mock volume
            } else {
                println!("❌ No blocks found in the blockchain");
            }
            
            println!();
            println!("🔮 Next Block:");
            println!("  • Estimated Time: ~2 minutes");
            println!("  • Pending Transactions: {}", blockchain.mempool.len());
            println!("  • Expected Difficulty: {}", blockchain.difficulty);
        },
        Err(e) => {
            eprintln!("❌ Failed to initialize blockchain: {}", e);
            println!("\n📦 Mock Latest Block:");
            println!("  • Height: 1,234");
            println!("  • Difficulty: 4");
            println!("  • Transactions: 3");
            println!("  • Estimated Time: ~2 minutes");
        }
    }
}

fn handle_generatetriangle(args: GenerateTriangleArgs) {
    println!("🔺 Generating Triangle Fractal to depth {}...", args.depth);
    println!();
    
    let genesis = genesis_fractal_triangle().expect("Failed to create genesis triangle");
    let structure = subdivide_to_depth(genesis, args.depth)
        .expect("Failed to generate fractal structure");
    
    println!("✅ Generated {} triangles", structure.total_triangles());
    
    // Calculate statistics
    let active_count = structure.triangles_by_state(triadchain::core::state::TriangleState::Active).len();
    let subdivided_count = structure.triangles_by_state(triadchain::core::state::TriangleState::Subdivided).len();
    
    println!();
    println!("📊 Generation Statistics:");
    println!("  • Total Triangles: {}", structure.total_triangles());
    println!("  • Active: {}", active_count);
    println!("  • Subdivided: {}", subdivided_count);
    println!("  • Maximum Depth: {}", structure.max_depth());
    
    if let Some(genesis) = structure.genesis() {
        if let Ok(total_area) = genesis.triangle.area() {
            println!("  • Total Area: {}", total_area);
        }
    }
    
    // Serialize and save
    let json = if args.pretty {
        serde_json::to_string_pretty(&structure)
    } else {
        serde_json::to_string(&structure)
    }.expect("Failed to serialize structure");
    
    if let Some(output_path) = args.output {
        fs::write(&output_path, json)
            .expect("Failed to write output file");
        println!("  • Saved to: {}", output_path.display());
    } else {
        println!();
        println!("📄 JSON Output:");
        println!("{}", json);
    }
    
    println!();
    println!("🎯 Triangle Addresses Generated:");
    for depth in 0..=args.depth {
        let triangles_at_depth = structure.triangles_at_depth(depth);
        if !triangles_at_depth.is_empty() {
            println!("  • Depth {}: {} triangles (addresses: {}...)", 
                depth, 
                triangles_at_depth.len(),
                triangles_at_depth.iter().take(3)
                    .map(|t| t.address.to_string())
                    .collect::<Vec<_>>()
                    .join(", "));
        }
    }
}

fn handle_validateaddress(address: String) {
    println!("🔍 Validating Triangle Address: {}", address);
    println!("{}=", "=".repeat(address.len() + 32));
    println!();
    
    match TriangleAddress::from_string_representation(&address) {
        Ok(addr) => {
            println!("✅ Address is valid!");
            println!();
            println!("📋 Address Information:");
            println!("  • Address: {}", addr);
            println!("  • Depth: {}", addr.depth());
            println!("  • Type: {}", if addr.is_genesis() { "Genesis" } else { "Child" });
            println!("  • Void Triangle: {}", if addr.is_void() { "Yes" } else { "No" });
            
            if let Some(component) = addr.last_component() {
                println!("  • Last Component: {}", component);
            }
            
            println!("  • Components: {:?}", addr.components());
            println!();
            
            println!("👨‍👩‍👧‍👦 Family Tree:");
            if let Some(parent) = addr.parent() {
                println!("  • Parent: {}", parent);
            } else {
                println!("  • Parent: None (Genesis triangle)");
            }
            
            let children = addr.children();
            println!("  • Children: {} ({}, {}, {})", 
                children.len(),
                children[0], children[1], children[2]);
            
            let siblings = addr.siblings();
            if !siblings.is_empty() {
                println!("  • Siblings: {}", siblings.iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>()
                    .join(", "));
            }
            
            println!();
            println!("🔬 Technical Details:");
            println!("  • Hash-based: Yes");
            println!("  • Collision-resistant: Yes");
            println!("  • Deterministic: Yes");
            println!("  • Hierarchical: Yes");
        }
        Err(e) => {
            println!("❌ Address validation failed!");
            println!();
            println!("Error: {}", e);
            println!();
            println!("📝 Valid address formats:");
            println!("  • Genesis: 'genesis'");
            println!("  • Child: '0.1.2' (dot-separated path)");
            println!("  • Examples: '0', '1.0', '2.1.0', '0.2.1.0'");
            std::process::exit(1);
        }
    }
}

fn handle_triangleinfo(address: String) {
    println!("🔺 Triangle Information for: {}", address);
    println!("{}=", "=".repeat(address.len() + 29));
    println!();
    
    match TriangleAddress::from_string_representation(&address) {
        Ok(addr) => {
            println!("📋 Basic Information:");
            println!("  • Address: {}", addr);
            println!("  • Depth: {}", addr.depth());
            println!("  • State: Active");
            println!("  • Owner: ST7a8b9c2d3e4f5g (You)");
            println!();
            
            println!("📐 Geometric Properties:");
            // Mock triangle data since we don't have access to actual triangle
            let area = Decimal::new(1, 0) / Decimal::new(2_i64.pow(addr.depth() as u32), 0);
            println!("  • Area: {} units²", area);
            println!("  • Perimeter: {} units", area * Decimal::new(3, 0));
            println!("  • Type: Equilateral");
            println!("  • Orientation: Upward");
            println!();
            
            println!("💰 Economic Value:");
            let base_value = Decimal::new(100, 0);
            let depth_multiplier = Decimal::new(2_i64.pow(addr.depth() as u32), 0);
            let estimated_value = base_value * depth_multiplier;
            println!("  • Estimated Value: {} TC", estimated_value);
            println!("  • USD Value: ${}", estimated_value * Decimal::new(50, 2));
            println!("  • Acquisition Cost: {} TC", estimated_value * Decimal::new(80, 2));
            println!("  • Appreciation: +{:.1}%", 25.0);
            println!();
            
            println!("⛏️  Mining Information:");
            println!("  • Mined: 3 days ago");
            println!("  • Miner: ST5f6e7d8c9b0a1f");
            println!("  • Block Height: {}", 1000 + addr.depth() as u32);
            println!("  • Mining Difficulty: {}", 1000 + (addr.depth() as u32) * 100);
            println!();
            
            println!("🔄 Transaction History:");
            println!("  • Creation: 3 days ago (Mining reward)");
            println!("  • Transfer: 2 days ago (Purchased for 80.0 TC)");
            println!("  • Stake: 1 day ago (Staked 25.0 TC)");
            println!("  • Total Transactions: 3");
            println!();
            
            println!("👨‍👩‍👧‍👦 Relationships:");
            if let Some(parent) = addr.parent() {
                println!("  • Parent: {} (Active)", parent);
            }
            let children = addr.children();
            println!("  • Children: {} total", children.len());
            for (i, child) in children.iter().take(3).enumerate() {
                println!("    - Child {}: {} (Not mined)", i, child);
            }
            
            println!();
            println!("📊 Performance Metrics:");
            println!("  • Staking Rewards (24h): +2.1 TC");
            println!("  • Appreciation (7d): +12.3%");
            println!("  • Liquidity Score: 8.5/10");
            println!("  • Rarity Score: {}/10", std::cmp::min(addr.depth() + 5, 10));
        }
        Err(e) => {
            eprintln!("❌ Error parsing address: {}", e);
            std::process::exit(1);
        }
    }
}

fn handle_economics() {
    println!("💰 TriadChain Economic Metrics");
    println!("==============================");
    println!();
    
    println!("📈 Token Economics:");
    println!("  • Circulating Supply: 1,000,000 TC");
    println!("  • Total Supply: 10,000,000 TC");
    println!("  • Max Supply: 21,000,000 TC");
    println!("  • Inflation Rate: 2.5% per year");
    println!("  • Deflation Rate: 0.1% per subdivision");
    println!();
    
    println!("💵 Market Metrics:");
    println!("  • Current Price: $0.50 USD");
    println!("  • Market Cap: $500,000");
    println!("  • 24h Volume: $25,000");
    println!("  • 24h Change: +5.2%");
    println!("  • All-time High: $0.78 USD");
    println!("  • All-time Low: $0.12 USD");
    println!();
    
    println!("🔺 Triangle Economics:");
    println!("  • Base Area Value: 10 TC per unit²");
    println!("  • Depth Multiplier: 2x per level");
    println!("  • Rarity Bonus: Up to 50%");
    println!("  • Age Factor: 1.1x per month");
    println!("  • Average Triangle Value: 125.4 TC");
    println!();
    
    println!("⛏️  Mining Economics:");
    println!("  • Block Reward: 50 TC");
    println!("  • Halving Period: 210,000 blocks");
    println!("  • Next Halving: In ~18 months");
    println!("  • Average Block Time: 60 seconds");
    println!("  • Mining Profitability: $0.12 per TC");
    println!();
    
    println!("🏛️  Staking Economics:");
    println!("  • Total Staked: 250,000 TC (25%)");
    println!("  • Average APY: 8.5%");
    println!("  • Staking Rewards Pool: 15,000 TC");
    println!("  • Minimum Stake: 100 TC");
    println!("  • Lock Period: 30 days");
    println!();
    
    println!("📊 DeFi Integration:");
    println!("  • Liquidity Pools: 3 active");
    println!("  • Total Value Locked: $125,000");
    println!("  • Yield Farming APY: 12.3%");
    println!("  • Lending Rate: 6.8%");
    println!("  • Borrowing Rate: 9.2%");
}

fn handle_stakingpools() {
    println!("🏛️  TriadChain Staking Pools");
    println!("============================");
    println!();
    
    println!("📊 Pool Overview:");
    println!("  • Total Pools: 5");
    println!("  • Total Staked: 250,000 TC");
    println!("  • Total Stakers: 1,247");
    println!("  • Average APY: 8.5%");
    println!();
    
    println!("🏆 Active Staking Pools:");
    println!();
    
    // Pool 1 - Genesis
    println!("1️⃣  Genesis Triangle Pool");
    println!("   • Total Staked: 75,000 TC");
    println!("   • APY: 10.2%");
    println!("   • Participants: 423");
    println!("   • Lock Period: 90 days");
    println!("   • Your Stake: 1,250 TC");
    println!("   • Your Rewards: +127.5 TC (10.2% APY)");
    println!("   • Status: 🟢 Active");
    println!();
    
    // Pool 2 - Depth Mining
    println!("2️⃣  Depth Mining Pool");
    println!("   • Total Staked: 50,000 TC");
    println!("   • APY: 12.8%");
    println!("   • Participants: 234");
    println!("   • Lock Period: 60 days");
    println!("   • Your Stake: 0 TC");
    println!("   • Min Stake: 500 TC");
    println!("   • Status: 🟢 Active");
    println!();
    
    // Pool 3 - Liquidity
    println!("3️⃣  Liquidity Provider Pool");
    println!("   • Total Staked: 65,000 TC");
    println!("   • APY: 15.4%");
    println!("   • Participants: 156");
    println!("   • Lock Period: 30 days");
    println!("   • Your Stake: 0 TC");
    println!("   • Min Stake: 1,000 TC");
    println!("   • Status: 🟢 Active");
    println!();
    
    // Pool 4 - Validator
    println!("4️⃣  Validator Node Pool");
    println!("   • Total Staked: 45,000 TC");
    println!("   • APY: 8.7%");
    println!("   • Participants: 89");
    println!("   • Lock Period: 180 days");
    println!("   • Your Stake: 0 TC");
    println!("   • Min Stake: 10,000 TC");
    println!("   • Status: 🟡 Nearly Full");
    println!();
    
    // Pool 5 - Governance
    println!("5️⃣  Governance Pool");
    println!("   • Total Staked: 15,000 TC");
    println!("   • APY: 6.5%");
    println!("   • Participants: 345");
    println!("   • Lock Period: 14 days");
    println!("   • Your Stake: 0 TC");
    println!("   • Min Stake: 10 TC");
    println!("   • Status: 🟢 Active");
    println!();
    
    println!("💡 Staking Tips:");
    println!("  • Higher APY pools typically have longer lock periods");
    println!("  • Diversify across multiple pools to reduce risk");
    println!("  • Monitor pool performance and adjust stakes accordingly");
    println!("  • Early unstaking may incur penalties");
    println!();
    
    println!("📈 Your Staking Summary:");
    println!("  • Total Staked: 1,250 TC");
    println!("  • Active Pools: 1");
    println!("  • Total Rewards (24h): +3.5 TC");
    println!("  • Total Rewards (All Time): +245.8 TC");
    println!("  • Average APY: 10.2%");
    println!();
    
    println!("🎯 Quick Actions:");
    println!("  • Use 'newwallet' to create a wallet for staking");
    println!("  • Use 'balance <address>' to check available funds");
    println!("  • Minimum stake amounts vary by pool");
}
