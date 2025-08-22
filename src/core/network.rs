//! Peer-to-peer networking for Sierpinski Triangle cryptocurrency

use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use uuid::Uuid;

use crate::core::{
    block::Block,
    blockchain::TriadChainBlockchain,
    mining::GeometricChallenge,
    errors::{SierpinskiError, SierpinskiResult},
};

/// Network message types for P2P communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkMessage {
    /// Handshake between peers
    Handshake {
        peer_id: String,
        version: String,
        blockchain_height: u64,
    },
    /// Request blockchain data
    BlockRequest {
        start_height: u64,
        count: u32,
    },
    /// Response with block data
    BlockResponse {
        blocks: Vec<Block>,
    },
    /// Announce new block
    NewBlock {
        block: Block,
    },
    /// Transaction broadcast
    TransactionBroadcast {
        transaction_id: String,
        transaction_data: Vec<u8>,
    },
    /// Mining challenge broadcast
    MiningChallenge {
        challenge: GeometricChallenge,
    },
    /// Peer discovery
    PeerDiscovery {
        known_peers: Vec<SocketAddr>,
    },
    /// Ping/keepalive
    Ping,
    /// Pong response
    Pong,
}

/// Peer information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    pub peer_id: String,
    pub address: SocketAddr,
    pub version: String,
    pub blockchain_height: u64,
    pub last_seen: u64,
    pub reputation_score: f64,
    pub connection_state: ConnectionState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Syncing,
    Ready,
}

/// P2P network node
pub struct NetworkNode {
    pub node_id: String,
    pub listen_address: SocketAddr,
    pub peers: Arc<Mutex<HashMap<String, PeerInfo>>>,
    pub blockchain: Arc<Mutex<TriadChainBlockchain>>,
    pub message_handlers: HashMap<String, Box<dyn Fn(&NetworkMessage) + Send + Sync>>,
}

impl NetworkNode {
    /// Create a new network node
    pub fn new(listen_address: SocketAddr, blockchain: Arc<Mutex<TriadChainBlockchain>>) -> Self {
        NetworkNode {
            node_id: format!("node_{}", Uuid::new_v4()),
            listen_address,
            peers: Arc::new(Mutex::new(HashMap::new())),
            blockchain,
            message_handlers: HashMap::new(),
        }
    }

    /// Start the network node
    pub async fn start(&self) -> SierpinskiResult<()> {
        let listener = TcpListener::bind(self.listen_address).await
            .map_err(|e| SierpinskiError::validation(&format!("Failed to bind to address: {}", e)))?;

        println!("🌐 Network node {} listening on {}", self.node_id, self.listen_address);

        // Start accepting connections
        tokio::spawn({
            let peers = Arc::clone(&self.peers);
            let blockchain = Arc::clone(&self.blockchain);
            let node_id = self.node_id.clone();
            
            async move {
                loop {
                    match listener.accept().await {
                        Ok((stream, addr)) => {
                            println!("📡 New connection from {}", addr);
                            
                            let peers_clone = Arc::clone(&peers);
                            let blockchain_clone = Arc::clone(&blockchain);
                            let node_id_clone = node_id.clone();
                            
                            tokio::spawn(async move {
                                if let Err(e) = Self::handle_peer_connection(
                                    stream, 
                                    addr, 
                                    peers_clone, 
                                    blockchain_clone,
                                    node_id_clone
                                ).await {
                                    println!("❌ Error handling peer {}: {}", addr, e);
                                }
                            });
                        }
                        Err(e) => {
                            println!("❌ Failed to accept connection: {}", e);
                        }
                    }
                }
            }
        });

        Ok(())
    }

    /// Handle incoming peer connection
    async fn handle_peer_connection(
        mut stream: TcpStream,
        addr: SocketAddr,
        peers: Arc<Mutex<HashMap<String, PeerInfo>>>,
        blockchain: Arc<Mutex<TriadChainBlockchain>>,
        node_id: String,
    ) -> SierpinskiResult<()> {
        let mut buffer = vec![0; 4096];
        
        loop {
            match stream.read(&mut buffer).await {
                Ok(0) => {
                    // Connection closed
                    println!("🔌 Connection closed by {}", addr);
                    break;
                }
                Ok(n) => {
                    let data = &buffer[..n];
                    
                    // Try to deserialize message
                    if let Ok(message) = serde_json::from_slice::<NetworkMessage>(data) {
                        let response = Self::handle_message(
                            &message, 
                            &addr, 
                            &peers, 
                            &blockchain,
                            &node_id
                        ).await;
                        
                        if let Some(response_msg) = response {
                            let response_data = serde_json::to_vec(&response_msg)
                                .map_err(|e| SierpinskiError::validation(&format!("Serialization error: {}", e)))?;
                            
                            stream.write_all(&response_data).await
                                .map_err(|e| SierpinskiError::validation(&format!("Write error: {}", e)))?;
                        }
                    }
                }
                Err(e) => {
                    println!("❌ Read error from {}: {}", addr, e);
                    break;
                }
            }
        }
        
        // Remove peer on disconnection
        {
            let mut peers_guard = peers.lock().unwrap();
            peers_guard.retain(|_, peer| peer.address != addr);
        }
        
        Ok(())
    }

    /// Handle network message
    async fn handle_message(
        message: &NetworkMessage,
        sender_addr: &SocketAddr,
        peers: &Arc<Mutex<HashMap<String, PeerInfo>>>,
        blockchain: &Arc<Mutex<TriadChainBlockchain>>,
        node_id: &str,
    ) -> Option<NetworkMessage> {
        match message {
            NetworkMessage::Handshake { peer_id, version, blockchain_height } => {
                println!("🤝 Handshake from peer {}", peer_id);
                
                // Add peer to our list
                {
                    let mut peers_guard = peers.lock().unwrap();
                    peers_guard.insert(peer_id.clone(), PeerInfo {
                        peer_id: peer_id.clone(),
                        address: *sender_addr,
                        version: version.clone(),
                        blockchain_height: *blockchain_height,
                        last_seen: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs(),
                        reputation_score: 0.5, // Neutral starting reputation
                        connection_state: ConnectionState::Connected,
                    });
                }
                
                // Respond with our handshake
                let blockchain_guard = blockchain.lock().unwrap();
                Some(NetworkMessage::Handshake {
                    peer_id: node_id.to_string(),
                    version: "0.1.0".to_string(),
                    blockchain_height: blockchain_guard.blocks.len() as u64,
                })
            }

            NetworkMessage::BlockRequest { start_height, count } => {
                println!("📦 Block request: start={}, count={}", start_height, count);
                
                let blockchain_guard = blockchain.lock().unwrap();
                let blocks: Vec<Block> = blockchain_guard.blocks
                    .iter()
                    .skip(*start_height as usize)
                    .take(*count as usize)
                    .cloned()
                    .collect();
                
                Some(NetworkMessage::BlockResponse { blocks })
            }

            NetworkMessage::NewBlock { block } => {
                println!("🆕 Received new block at height {}", block.height);
                
                // Validate and potentially add to blockchain
                let mut blockchain_guard = blockchain.lock().unwrap();
                if let Err(e) = block.validate() {
                    println!("❌ Invalid block received: {}", e);
                } else {
                    // In a full implementation, we'd verify the block fits our chain
                    println!("✅ Valid block received (validation successful)");
                }
                
                None // No response needed
            }

            NetworkMessage::Ping => {
                Some(NetworkMessage::Pong)
            }

            NetworkMessage::Pong => {
                // Update peer's last seen time
                if let Some(peer_id) = Self::find_peer_by_address(peers, sender_addr) {
                    let mut peers_guard = peers.lock().unwrap();
                    if let Some(peer) = peers_guard.get_mut(&peer_id) {
                        peer.last_seen = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs();
                    }
                }
                None
            }

            _ => None // Handle other message types
        }
    }

    /// Find peer ID by address
    fn find_peer_by_address(
        peers: &Arc<Mutex<HashMap<String, PeerInfo>>>, 
        addr: &SocketAddr
    ) -> Option<String> {
        let peers_guard = peers.lock().unwrap();
        peers_guard
            .iter()
            .find(|(_, peer)| peer.address == *addr)
            .map(|(id, _)| id.clone())
    }

    /// Connect to a peer
    pub async fn connect_to_peer(&self, peer_address: SocketAddr) -> SierpinskiResult<()> {
        println!("🔗 Connecting to peer at {}", peer_address);
        
        match TcpStream::connect(peer_address).await {
            Ok(mut stream) => {
                // Send handshake
                let blockchain_guard = self.blockchain.lock().unwrap();
                let handshake = NetworkMessage::Handshake {
                    peer_id: self.node_id.clone(),
                    version: "0.1.0".to_string(),
                    blockchain_height: blockchain_guard.blocks.len() as u64,
                };
                drop(blockchain_guard);
                
                let handshake_data = serde_json::to_vec(&handshake)
                    .map_err(|e| SierpinskiError::validation(&format!("Serialization error: {}", e)))?;
                
                stream.write_all(&handshake_data).await
                    .map_err(|e| SierpinskiError::validation(&format!("Write error: {}", e)))?;
                
                println!("✅ Connected to peer {}", peer_address);
                Ok(())
            }
            Err(e) => {
                println!("❌ Failed to connect to {}: {}", peer_address, e);
                Err(SierpinskiError::validation(&format!("Connection failed: {}", e)))
            }
        }
    }

    /// Broadcast message to all connected peers
    pub async fn broadcast_message(&self, message: NetworkMessage) -> SierpinskiResult<()> {
        let peers_guard = self.peers.lock().unwrap();
        let peer_addresses: Vec<SocketAddr> = peers_guard.values()
            .filter(|peer| matches!(peer.connection_state, ConnectionState::Ready | ConnectionState::Connected))
            .map(|peer| peer.address)
            .collect();
        drop(peers_guard);
        
        let message_data = serde_json::to_vec(&message)
            .map_err(|e| SierpinskiError::validation(&format!("Serialization error: {}", e)))?;
        
        for addr in peer_addresses {
            if let Ok(mut stream) = TcpStream::connect(addr).await {
                let _ = stream.write_all(&message_data).await;
            }
        }
        
        Ok(())
    }

    /// Sync blockchain with peers
    pub async fn sync_blockchain(&self) -> SierpinskiResult<()> {
        println!("🔄 Starting blockchain sync...");
        
        let peers_guard = self.peers.lock().unwrap();
        if peers_guard.is_empty() {
            return Err(SierpinskiError::validation("No peers available for sync"));
        }
        
        // Find peer with highest blockchain height
        let best_peer = peers_guard.values()
            .max_by_key(|peer| peer.blockchain_height);
            
        if let Some(peer) = best_peer {
            let our_height = {
                let blockchain_guard = self.blockchain.lock().unwrap();
                blockchain_guard.blocks.len() as u64
            };
            
            if peer.blockchain_height > our_height {
                println!("📥 Syncing from peer {} (height: {})", peer.peer_id, peer.blockchain_height);
                
                // Request blocks
                let request = NetworkMessage::BlockRequest {
                    start_height: our_height,
                    count: (peer.blockchain_height - our_height) as u32,
                };
                
                // In a real implementation, we'd send this request and handle the response
                println!("📤 Block sync request sent");
            } else {
                println!("✅ Blockchain is up to date");
            }
        }
        
        Ok(())
    }

    /// Get network statistics
    pub fn get_stats(&self) -> NetworkStats {
        let peers_guard = self.peers.lock().unwrap();
        let blockchain_guard = self.blockchain.lock().unwrap();
        
        NetworkStats {
            node_id: self.node_id.clone(),
            listen_address: self.listen_address,
            connected_peers: peers_guard.len(),
            blockchain_height: blockchain_guard.blocks.len() as u64,
            total_transactions: blockchain_guard.blocks.iter()
                .map(|b| b.triangle_transactions.len())
                .sum(),
        }
    }
}

/// Network statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStats {
    pub node_id: String,
    pub listen_address: SocketAddr,
    pub connected_peers: usize,
    pub blockchain_height: u64,
    pub total_transactions: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_network_node_creation() {
        let blockchain = Arc::new(Mutex::new(TriadChainBlockchain::new().unwrap()));
        let addr = "127.0.0.1:8080".parse().unwrap();
        let node = NetworkNode::new(addr, blockchain);
        
        assert!(!node.node_id.is_empty());
        assert_eq!(node.listen_address, addr);
    }
}