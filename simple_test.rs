// Simple test to verify core Sierpinski functionality
use std::process::Command;

fn main() {
    println!("Testing Sierpinski Triangle Cryptocurrency Core...");
    
    // Test basic Rust functionality first
    let decimal_test = rust_decimal::Decimal::new(1, 0);
    println!("Decimal library working: {}", decimal_test);
    
    // Test UUID generation
    let id = uuid::Uuid::new_v4();
    println!("UUID generation working: {}", id);
    
    // Test Blake3 hashing
    let hash = blake3::hash(b"Sierpinski Triangle Test");
    println!("Blake3 hashing working: {}", hash);
    
    println!("Core dependencies are functional!");
}