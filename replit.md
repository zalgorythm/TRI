# Overview

This project is a blockchain-based cryptocurrency system built in Rust that uses the Sierpinski Triangle fractal as its core mathematical foundation. The system implements a unique geometric proof-of-work consensus mechanism where miners perform triangle subdivisions following the Sierpinski Triangle pattern. The architecture combines traditional blockchain concepts with fractal geometry, creating a novel cryptocurrency that uses geometric operations as the basis for mining and transaction validation.

# User Preferences

Preferred communication style: Simple, everyday language.

# System Architecture

## Core Architecture
The system follows a modular Rust library structure with distinct layers for geometric operations, blockchain functionality, and network communications. The core is built around fractal triangle management with a hierarchical addressing system that treats triangle subdivisions as blockchain operations.

## Geometric Foundation
The architecture centers on precise coordinate handling using fixed-point decimals (rust_decimal crate) to ensure deterministic geometric calculations across all nodes. Triangle states are managed through an enum system (Genesis, Active, Subdivided, Void) with strict validation rules for geometric integrity.

## Blockchain Design
Uses a custom block structure that contains triangle transaction vectors and geometric proofs rather than traditional financial transactions. The consensus mechanism is based on geometric proof-of-work where miners must perform valid Sierpinski Triangle subdivisions with increasing complexity.

## Mining System
Implements a unique geometric proof-of-work algorithm where mining difficulty adjusts based on triangle subdivision complexity and network hashrate. Miners compete to find valid triangle subdivisions that meet geometric and cryptographic requirements.

## Storage Strategy
Designed with abstraction layers using Rust traits to support multiple storage backends. Initial implementation uses in-memory storage with plans for database integration, optimized for geometric data structures and triangle hierarchy queries.

## Network Protocol
Custom peer-to-peer protocol designed for efficient synchronization of triangle data and geometric proofs. Includes specialized message types for triangle operations and peer reputation tracking based on geometric proof quality.

# External Dependencies

## Core Rust Crates
- `rust_decimal`: Provides precise fixed-point decimal arithmetic for deterministic geometric calculations
- `ed25519-dalek`: Implements digital signatures for transaction authentication
- `thiserror`: Structured error handling across all geometric and blockchain operations
- `serde`: Serialization framework for network communication and data persistence

## Cryptographic Libraries
- SHA-256 hashing algorithms for block and transaction integrity
- Ed25519 elliptic curve cryptography for wallet operations and transaction signing
- Custom geometric hash functions for triangle-specific operations

## Network Dependencies
- TCP/IP networking for peer-to-peer communication
- Binary serialization protocols for efficient triangle data transmission
- Future integration planned for distributed storage systems and database backends

## Development Tools
- Cargo build system for Rust project management
- Standard Rust testing framework for geometric validation and blockchain testing
- Documentation generation tools for API reference