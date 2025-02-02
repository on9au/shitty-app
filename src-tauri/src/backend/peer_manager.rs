use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use base64::{prelude::BASE64_STANDARD, Engine};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{tcp::OwnedReadHalf, TcpListener, TcpStream},
    sync::{mpsc, oneshot, Mutex},
};
use tracing::{debug, error, info, trace, warn};

use crate::js_api::backend_event::{BackendEvent, ConnectionCloseOrBroken, ConnectionInfo};

use super::protocol::{DisconnectRequest, Message};

/// Peer Manager
///
/// Manages the peers that the application is connected to.
#[derive(Debug, Clone)]
pub struct PeerManager {
    /// List of connected peers
    /// Hashmap of peer's IP/Socket address to their mpsc sender
    pub(crate) active_peers: Arc<Mutex<HashMap<SocketAddr, Peer>>>,
    /// Reference to the backend event sender
    pub(crate) backend_event_tx: mpsc::Sender<BackendEvent>,
    /// Shutdown one-shot sender. If None, the PeerManager has been shutdown.
    pub(crate) shutdown_tx: Arc<Mutex<Option<oneshot::Sender<()>>>>,
}

/// Peer
///
/// Represents a peer that the application is connected to.
#[derive(Debug)]
pub struct Peer {
    /// IP/Socket address of the peer
    pub addr: SocketAddr,
    /// State of the peer
    pub state: PeerState,
    /// The sender to send messages to the peer
    pub tx: mpsc::Sender<Message>,
}

impl Drop for Peer {
    fn drop(&mut self) {
        match &self.state {
            PeerState::Connected { .. } => {
                info!("Peer disconnected during authentication: {:?}", self);
            }
            PeerState::Authenticated { .. } => {
                info!("Peer improperly disconnected: {:?}", self);
            }
            PeerState::Disconnecting { reason, .. } => {
                info!(
                    "Peer successfully disconnected: {:?} Reason: {}",
                    self,
                    reason.clone().unwrap_or("None".to_string())
                );
            }
        }
    }
}

/// Peer State
///
/// Represents the state of a peer.
#[derive(Debug)]
pub enum PeerState {
    /// Connected via TCP, but not yet authenticated
    Connected {
        /// Peer information (if connect request has been received)
        peer_info: Option<PeerInfo>,
    },
    /// Authenticated and ready to send/receive messages
    Authenticated {
        /// Peer information
        peer_info: PeerInfo,
    },
    /// Intending to disconnect
    Disconnecting {
        /// The reason for disconnection
        reason: Option<String>,
        /// Peer information
        peer_info: PeerInfo,
    },
}

#[derive(Debug, Clone)]
pub struct PeerInfo {
    /// The name of the peer
    pub name: String,
    /// The ECDSA public key of the peer
    pub ecdsa_public_key: Vec<u8>,
    /// The Backend version of the peer
    pub backend_version: String,
}

impl PeerManager {
    /// Create a new PeerManager
    pub fn new(backend_event_tx: mpsc::Sender<BackendEvent>) -> Self {
        Self {
            active_peers: Arc::new(Mutex::new(HashMap::new())),
            backend_event_tx,
            shutdown_tx: Arc::new(Mutex::new(None)),
        }
    }

    /// Gracefully shutdown the PeerManager
    ///
    /// Should not be called before `start` has been called, else it returns immediately.
    pub async fn shutdown(&self) {
        info!("Shutting down PeerManager");

        // Send a shutdown signal to the PeerManager
        if let Some(shutdown_tx) = self.shutdown_tx.lock().await.take() {
            shutdown_tx.send(()).ok();
            // self.shutdown_tx is now = None
        } else {
            warn!("PeerManager has already been shutdown, or never started. Aborting shutdown.");
            return;
        }

        let mut active_peers = self.active_peers.lock().await;
        for (peer_addr, peer) in active_peers.drain() {
            // Send an ImmediateConnectionClose message to the peer
            peer.tx
                .send(Message::ImmediateConnectionClose(DisconnectRequest {
                    message: "Peer is shutting down".to_string().into(),
                }))
                .await
                .ok();

            // Drop the peer
            self.drop_peer(peer_addr, None).await;
        }

        info!("PeerManager has been shutdown");
    }

    /// Is the PeerManager running?
    pub async fn is_running(&self) -> bool {
        self.shutdown_tx.lock().await.is_some()
    }

    /// Begin listening for incoming connections from new peers
    pub async fn start(&self, listen_addr: &str) -> Result<(), Box<dyn std::error::Error>> {
        let listener = TcpListener::bind(listen_addr).await?;
        let (shutdown_tx, mut shutdown_rx) = oneshot::channel(); // Create a shutdown signal

        // Set the shutdown signal
        *self.shutdown_tx.lock().await = Some(shutdown_tx);

        info!("Listening for incoming connections on {}", listen_addr);

        // Accept incoming connections
        // Once accepted, spawn a new task to handle the connection
        loop {
            tokio::select! {
                result = listener.accept() => {
                    match result {
                        Ok((stream, peer_addr)) => {
                            info!("Accepted connection from {}", peer_addr);
                            let manager = self.clone();
                            tokio::spawn(async move {
                                manager.handle_connection(stream, peer_addr).await;
                            });
                        }
                        Err(e) => {
                            error!("Failed to accept connection: {}", e);
                        }
                    }
                }

                _ = &mut shutdown_rx => {
                        info!("PeerManager will not accept new connections. Goodbye!");
                        break Ok(());
                }
            }
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
                    addr: peer_addr,
                    state: PeerState::Connected { peer_info: None },
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
                                    warn!("Failed to flush: {}", e);
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
            match tokio::time::timeout(tokio::time::Duration::from_secs(30), stream.read(&mut buf))
                .await
            {
                // EOF, connection closed
                Ok(Ok(0)) => {
                    // EOF, connection closed
                    // Check if this was a normal close or a broken pipe

                    self.drop_peer(peer_addr, None).await;

                    break 'recv;
                }
                // Deserialize message and handle it
                Ok(Ok(n)) => {
                    let message: Message = match bincode::deserialize(&buf[..n]) {
                        Ok(message) => message,
                        Err(e) => {
                            warn!(
                                "Failed to deserialize peer message: {}. Closing connection. Err: {}",
                                peer_addr, e
                            );
                            trace!(
                                "Raw contents of message from {}: {:?}",
                                peer_addr,
                                &buf[..n]
                            );

                            // Remove peer from active peers to drop the sender
                            self.drop_peer(
                                peer_addr,
                                format!("Failed to deserialize peer message: {}", e).into(),
                            )
                            .await;

                            break 'recv;
                        }
                    };
                    self.handle_message(message, peer_addr).await;
                }
                // Error reading from stream
                Ok(Err(e)) => {
                    warn!(
                        "Failed to read buffer from peer: {}. Closing connection. Err: {}",
                        peer_addr, e
                    );

                    self.drop_peer(
                        peer_addr,
                        format!("Failed to read buffer from peer: {}", e).into(),
                    )
                    .await;

                    break 'recv;
                }
                // Timeout reading from stream
                Err(_) => {
                    warn!(
                        "Timeout reading from peer {}. Closing connection.",
                        peer_addr
                    );

                    self.drop_peer(peer_addr, "Timeout reading from peer.".to_string().into())
                        .await;

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
            Message::KeepAlive => {
                self.handle_keep_alive(peer_addr).await;
            }
            Message::ConnectRequest(connection_info) => {
                self.handle_connect_request(connection_info, peer_addr)
                    .await;
            }
            Message::ConnectResponse(connection_response) => {
                self.handle_connect_response(connection_response, peer_addr)
                    .await;
            }
            Message::DisconnectRequest(disconnect_request) => {
                self.handle_disconnect_request(disconnect_request, peer_addr)
                    .await;
            }
            Message::DisconnectAck => {
                self.handle_disconnect_ack(peer_addr).await;
            }
            Message::ImmediateConnectionClose(_disconnect_request) => todo!(),
            Message::FileOfferRequest(_file_offer) => todo!(),
            Message::FileOfferResponse(_file_offer_response) => todo!(),
            Message::FileChunk(_file_chunk) => todo!(),
            Message::FileChunkAck(_file_chunk_ack) => todo!(),
            Message::FileDone(_file_done) => todo!(),
            Message::FileDoneResult(_file_done_result) => todo!(),
        }
    }

    /// Drop a peer.
    /// Notify frontend if the peer was authenticated.
    ///
    /// If peer's state is `Authenticated`, send a `ConnectionBroken` event to the frontend.
    /// If peer's state is `Disconnecting`, send a `ConnectionClose` event to the frontend.
    /// If peer's state is `Connected`, do not send any event to the frontend.
    ///
    /// Message is optional, however will always override the reason for disconnection.
    pub async fn drop_peer(&self, peer_addr: SocketAddr, message: Option<String>) {
        let mut active_peers = self.active_peers.lock().await;
        let removed_peer = active_peers.remove(&peer_addr);
        if let Some(removed_peer) = removed_peer {
            match &removed_peer.state {
                PeerState::Authenticated {
                    peer_info:
                        PeerInfo {
                            name,
                            ecdsa_public_key,
                            backend_version,
                        },
                } => {
                    self.backend_event_tx
                        .send(BackendEvent::ConnectionBroken(ConnectionCloseOrBroken {
                            connection_info: ConnectionInfo {
                                name: name.to_string(),
                                ip: peer_addr.ip().to_string(),
                                backend_version: backend_version.to_string(),
                                identitiy: BASE64_STANDARD.encode(ecdsa_public_key),
                            },
                            message,
                        }))
                        .await
                        .expect("Failed to send ConnectionBroken event to the frontend");
                }
                PeerState::Disconnecting { peer_info, reason } => {
                    self.backend_event_tx
                        .send(BackendEvent::ConnectionClose(ConnectionCloseOrBroken {
                            connection_info: ConnectionInfo {
                                name: peer_info.name.clone(),
                                ip: peer_addr.ip().to_string(),
                                backend_version: peer_info.backend_version.clone(),
                                identitiy: BASE64_STANDARD.encode(&peer_info.ecdsa_public_key),
                            },
                            message: {
                                if let Some(message) = message {
                                    Some(message)
                                } else {
                                    reason.clone()
                                }
                            },
                        }))
                        .await
                        .expect("Failed to send ConnectionClose event to the frontend");
                }
                PeerState::Connected { .. } => {}
            }
        }
    }
}
