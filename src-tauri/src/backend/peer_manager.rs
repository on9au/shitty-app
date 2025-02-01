use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{tcp::OwnedReadHalf, TcpListener, TcpStream},
    sync::{mpsc, Mutex},
};
use tracing::{debug, info, trace, warn};

use crate::js_api::backend_event::BackendEvent;

use super::protocol::Message;

/// Peer Manager
///
/// Manages the peers that the application is connected to.
#[derive(Debug, Clone)]
pub struct PeerManager {
    /// List of connected peers
    /// Hashmap of peer's IP/Socket address to their mpsc sender
    active_peers: Arc<Mutex<HashMap<SocketAddr, Peer>>>,
    /// Reference to the backend event sender
    backend_event_tx: mpsc::Sender<BackendEvent>,
}

/// Peer
///
/// Represents a peer that the application is connected to.
#[derive(Debug)]
pub struct Peer {
    /// State of the peer
    pub state: PeerState,
    /// The sender to send messages to the peer
    pub tx: mpsc::Sender<Message>,
}

/// Peer State
///
/// Represents the state of a peer.
#[derive(Debug)]
pub enum PeerState {
    /// Connected via TCP, but not yet authenticated
    Connected,
    /// Authenticated and ready to send/receive messages
    Authenticated {
        /// The name of the peer
        name: String,
        /// The ECDSA public key of the peer
        ecdsa_public_key: Vec<u8>,
        /// The Backend version of the peer
        backend_version: String,
    },
}

impl PeerManager {
    /// Create a new PeerManager
    pub fn new(backend_event_tx: mpsc::Sender<BackendEvent>) -> Self {
        Self {
            active_peers: Arc::new(Mutex::new(HashMap::new())),
            backend_event_tx,
        }
    }

    /// Begin listening for incoming connections from new peers
    pub async fn start(&self, listen_addr: &str) -> Result<(), Box<dyn std::error::Error>> {
        let listener = TcpListener::bind(listen_addr).await?;

        info!("Listening for incoming connections on {}", listen_addr);

        // Accept incoming connections
        // Once accepted, spawn a new task to handle the connection
        loop {
            let (stream, peer_addr) = listener.accept().await?;
            let manager = self.clone();

            info!("Accepted connection from peer {}", peer_addr);

            tokio::spawn(async move {
                manager.handle_connection(stream, peer_addr).await;
            });
        }
    }

    /// Handle connections from a peer
    async fn handle_connection(&self, stream: TcpStream, peer_addr: SocketAddr) {
        let (tx, mut rx) = mpsc::channel(32);
        let (reader, mut writer) = stream.into_split();

        // Insert sender into active peers
        {
            let mut active_peers = self.active_peers.lock().await;
            active_peers.insert(
                peer_addr,
                Peer {
                    state: PeerState::Connected,
                    tx,
                },
            );
        }

        // Spawn a task to read from the peer
        let manager_clone = self.clone();
        tokio::spawn(async move {
            manager_clone.read_messages(reader, peer_addr).await;
        });

        // Spawn a task to write to the peer
        tokio::spawn(async move {
            while let Some(message) = rx.recv().await {
                match &message {
                    Message::FileChunk(chunk) => {
                        info!(
                            "Sending FileChunk: ID={} Chunk={:4}/{:4}",
                            chunk.unique_id,
                            chunk.chunk_id,
                            chunk.chunk_len - 1
                        );
                    }
                    _ => info!("Sending control message: {:?}", message),
                }

                match bincode::serialize(&message) {
                    Ok(bytes) => {
                        if writer.writable().await.is_ok() {
                            if let Err(e) = writer.write_all(&bytes).await {
                                warn!("Failed to send message: {}", e);
                                break;
                            }

                            // Force flush periodically to improve real-time file transfer
                            if matches!(message, Message::FileChunk(_)) {
                                if let Err(e) = writer.flush().await {
                                    eprintln!("Failed to flush: {}", e);
                                    break;
                                }
                            }
                        }
                    }
                    Err(e) => warn!("Serialization failed: {}", e),
                }
            }
        });
    }

    /// Read messages from a peer
    async fn read_messages(&self, mut stream: OwnedReadHalf, peer_addr: SocketAddr) {
        let mut buf: [u8; 4096] = [0; 4096]; // 4KB buffer

        // Read loop
        'recv: loop {
            match stream.read(&mut buf).await {
                // EOF, connection closed
                Ok(0) => {
                    // EOF, connection closed
                    // Remove peer from active peers
                    info!("Peer {} disconnected", peer_addr);
                    {
                        let mut active_peers = self.active_peers.lock().await;
                        active_peers.remove(&peer_addr);
                    }
                    break 'recv;
                }
                // Deserialize message and handle it
                Ok(n) => {
                    let message: Message = bincode::deserialize(&buf[..n])
                        .map_err(|e| {
                            warn!(
                                "Failed to deserialize peer message: {}. Closing connection. Err: {}",
                                peer_addr, e
                            );
                            trace!("Raw contents of message from {}: {:?}", peer_addr, &buf[..n]);
                            e
                        })
                        .expect("Failed to deserialize message");
                    self.handle_message(message, peer_addr).await;
                }
                // Error reading from stream
                Err(e) => {
                    warn!(
                        "Failed to read from peer: {}. Closing connection. Err: {}",
                        peer_addr, e
                    );
                    {
                        let mut active_peers = self.active_peers.lock().await;
                        active_peers.remove(&peer_addr);
                    }
                    break 'recv;
                }
            }
        }
    }

    // TODO: Implement message handling
    /// Handle a message from a peer
    async fn handle_message(&self, message: Message, peer_addr: SocketAddr) {
        debug!("Received message from peer {}: {:?}", peer_addr, message);
        match message {
            Message::InvalidMessage(_) => todo!(),
            Message::ConnectRequest(_connection_info) => todo!(),
            Message::ConnectResponse(_) => todo!(),
            Message::DisconnectRequest(_) => todo!(),
            Message::DisconnectAck => todo!(),
            Message::FileOfferRequest(_file_offer) => todo!(),
            Message::FileOfferResponse { accept: _ } => todo!(),
            Message::FileChunk(_file_chunk) => todo!(),
            Message::FileChunkAck(_file_chunk_ack) => todo!(),
            Message::FileDone(_file_done) => todo!(),
            Message::FileDoneResult {
                success: _,
                message: _,
            } => todo!(),
        }
    }
}
