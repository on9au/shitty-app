use std::{collections::HashMap, sync::Arc};

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
    /// Hashmap of peer's IP address to their mpsc sender
    active_peers: Arc<Mutex<HashMap<String, mpsc::Sender<Message>>>>,
    /// Reference to the backend event sender
    backend_event_tx: mpsc::Sender<BackendEvent>,
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
            let peer_addr = peer_addr.to_string();
            let manager = self.clone();

            info!("Accepted connection from peer {}", peer_addr);

            tokio::spawn(async move {
                manager.handle_connection(stream, peer_addr).await;
            });
        }
    }

    /// Handle connections from a peer
    async fn handle_connection(&self, stream: TcpStream, peer_addr: String) {
        let (tx, mut rx) = mpsc::channel(100);
        let (reader, mut writer) = stream.into_split();

        // Insert sender into active peers
        {
            let mut active_peers = self.active_peers.lock().await;
            active_peers.insert(peer_addr.clone(), tx);
        }

        // Spawn a task to read from the peer
        let manager_clone = self.clone();
        tokio::spawn(async move {
            manager_clone.read_messages(reader, peer_addr.clone()).await;
        });

        // Spawn a task to write to the peer
        tokio::spawn(async move {
            while let Some(message) = rx.recv().await {
                let bytes = bincode::serialize(&message).expect("Failed to serialize message");
                writer.writable().await.unwrap();
                writer
                    .write_all(&bytes)
                    .await
                    .expect("Failed to write to peer");
            }
        });
    }

    /// Read messages from a peer
    async fn read_messages(&self, mut stream: OwnedReadHalf, peer_addr: String) {
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
                    self.handle_message(message, peer_addr.clone()).await;
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
    async fn handle_message(&self, message: Message, peer_addr: String) {
        debug!("Received message from peer {}: {:?}", peer_addr, message);
        match message {
            Message::InvalidMessage(_) => todo!(),
            Message::ConnectRequest(_connection_info) => todo!(),
            Message::ConnectResponse {
                success: _,
                message: _,
            } => todo!(),
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
