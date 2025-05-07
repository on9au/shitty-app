use std::{collections::HashMap, net::SocketAddr, sync::Arc};

// use base64::{Engine, prelude::BASE64_STANDARD};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream, tcp::OwnedReadHalf},
    sync::{Mutex, mpsc, oneshot},
};
use tracing::{debug, error, info, trace, warn};
use uuid::Uuid;

use crate::js_api::backend_event::{BackendEvent, ConnectionCloseOrBroken, ConnectionInfo};

use super::protocol::{BINCODE_CONFIG, DisconnectRequest, MAX_MESSAGE_SIZE, Message};

/// Peer Manager
///
/// Manages the peers that the application is connected to.
#[derive(Debug, Clone)]
pub struct PeerManager {
    /// List of connected peers
    /// Hashmap of peer's IP/Socket address to their mpsc sender
    pub(crate) active_peers: Arc<Mutex<HashMap<SocketAddr, Peer>>>,
    /// File transfer state, keyed by File Transfer unique_id
    pub(crate) active_transfers: Arc<Mutex<HashMap<Uuid, FileTransferState>>>,
    /// Reference to the backend event sender
    pub(crate) backend_event_tx: mpsc::Sender<BackendEvent>,
    /// Shutdown one-shot sender. If None, the PeerManager has been shutdown.
    pub(crate) shutdown_tx: Arc<Mutex<Option<oneshot::Sender<()>>>>,
}

/// File Transfer Direction
#[derive(Debug, PartialEq)]
pub enum FileTransferDirection {
    Sending {
        /// The file path of the file being sent
        file_path: String,
    },
    Receiving,
}

/// Represents the state of a file transfer.
#[derive(Debug)]
pub struct FileTransferState {
    /// Unique ID of the file transfer
    pub unique_id: Uuid,
    /// IP/Socket address of the peer
    pub peer_addr: std::net::SocketAddr,
    /// Direction of the file transfer
    pub direction: FileTransferDirection,
    /// The file handle of the file being transferred
    pub filename: String,
    /// The size of the file being transferred
    pub total_size: u64,
    /// The number of bytes transferred so far
    pub bytes_transferred: u64,
    /// The length of the chunks being transferred
    pub chunk_len: u64,
    /// The status of the file transfer
    pub status: FileTransferStatus,
    // Optionally: file handles, checksums, etc.
}

/// File Transfer Status
#[derive(Debug)]
pub enum FileTransferStatus {
    /// Waiting for peer's response (we do not accept file chunks yet)
    WaitingForPeerResponse,
    /// The file transfer is in progress (we can accept file chunks now)
    InProgress {
        /// Handle to file being transferred
        file_handle: Arc<tokio::fs::File>,
    },
    /// The file transfer is completed
    Completed,
    /// The file transfer is cancelled (but was accepted)
    Cancelled,
    /// The file transfer was rejected (not accepted)
    Rejected,
    /// The file transfer failed
    Error(String),
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
    // /// The ECDSA public key of the peer
    // pub ecdsa_public_key: Vec<u8>,
    /// The Backend version of the peer
    pub backend_version: String,
}

impl PeerInfo {
    /// Convert PeerInfo to ConnectionInfo
    pub fn into_connection_info(&self, peer_addr: SocketAddr) -> ConnectionInfo {
        ConnectionInfo {
            name: self.name.clone(),
            ip: peer_addr.ip().to_string(),
            backend_version: self.backend_version.clone(),
            // identitiy: BASE64_STANDARD.encode(&self.ecdsa_public_key),
        }
    }
}

impl PeerManager {
    /// Create a new PeerManager
    pub fn new(backend_event_tx: mpsc::Sender<BackendEvent>) -> Self {
        Self {
            active_peers: Arc::new(Mutex::new(HashMap::new())),
            active_transfers: Arc::new(Mutex::new(HashMap::new())),
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
        match self.shutdown_tx.lock().await.take() {
            Some(shutdown_tx) => {
                shutdown_tx.send(()).ok();
                // self.shutdown_tx is now = None
            }
            _ => {
                warn!(
                    "PeerManager has already been shutdown, or never started. Aborting shutdown."
                );
                return;
            }
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
    pub async fn start(
        &self,
        listen_addr: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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

    /// Connect to a peer
    pub async fn connect(
        &self,
        peer_addr: SocketAddr,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Check if we are shut down
        if self.shutdown_tx.lock().await.is_none() {
            warn!("PeerManager is shut down. Cannot connect to peer.");
            return Err("PeerManager is shut down".into());
        }

        // Check if we are already connected to the peer
        if self.active_peers.lock().await.contains_key(&peer_addr) {
            warn!("Already connected to peer {}", peer_addr);
            return Err("Already connected to peer".into());
        }

        // Connect to the peer
        let stream = TcpStream::connect(peer_addr).await?;

        info!("Connection accepted from {}", peer_addr);
        let manager = self.clone();
        tokio::spawn(async move {
            manager.handle_connection(stream, peer_addr).await;
        });

        Ok(())
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
        let manager_clone_clone = self.clone();
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

                match bincode::encode_to_vec(&message, *BINCODE_CONFIG) {
                    Ok(bytes) => {
                        if writer.writable().await.is_ok() {
                            // Check if we are sending a message larger than the maximum size
                            if bytes.len() > MAX_MESSAGE_SIZE {
                                warn!(
                                    "We are trying to send a message to peer {} larger than the maximum size of {} bytes. THIS IS A BUG!",
                                    peer_addr, MAX_MESSAGE_SIZE
                                );

                                // Remove peer from active peers to drop the sender
                                manager_clone_clone
                                    .drop_peer(
                                        peer_addr,
                                        Some(format!(
                                            "We are trying to send a message to peer {} larger than the maximum size of {} bytes. THIS IS A BUG!",
                                            peer_addr, MAX_MESSAGE_SIZE
                                        )),
                                    )
                                    .await;

                                break;
                            }

                            let len = (bytes.len() as u32).to_be_bytes();

                            // Write the length of the message
                            if let Err(e) = writer.write_all(&len).await {
                                warn!("Failed to send message length: {}", e);

                                // Remove peer from active peers to drop the sender
                                manager_clone_clone
                                    .drop_peer(
                                        peer_addr,
                                        format!("Failed to send message len: {}", e).into(),
                                    )
                                    .await;

                                break;
                            }

                            // Write the data of the message
                            if let Err(e) = writer.write_all(&bytes).await {
                                warn!("Failed to send message: {}", e);

                                // Remove peer from active peers to drop the sender
                                manager_clone_clone
                                    .drop_peer(
                                        peer_addr,
                                        format!("Failed to send message data: {}", e).into(),
                                    )
                                    .await;

                                break;
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
        let mut len_buf = [0u8; 4]; // 4-byte length buffer

        'recv: loop {
            // Read the length of the message
            match stream.read_exact(&mut len_buf).await {
                Ok(_) => {
                    let len = u32::from_be_bytes(len_buf) as usize;

                    // Check if len is valid BEFORE allocating the buffer (prevent DoS)
                    if len > MAX_MESSAGE_SIZE {
                        warn!(
                            "Peer {} sent a message larger than the maximum size of {} bytes. Closing connection.",
                            peer_addr, MAX_MESSAGE_SIZE
                        );

                        debug!("Peer {} sent a len header with value: {}.", peer_addr, len);

                        // Remove peer from active peers to drop the sender
                        self.drop_peer(
                            peer_addr,
                            Some(format!(
                                "Peer sent a message larger than the maximum size of {} bytes",
                                MAX_MESSAGE_SIZE
                            )),
                        )
                        .await;

                        break 'recv;
                    }

                    let mut buf = vec![0u8; len]; // variable length buffer

                    // Read the message
                    match stream.read_exact(&mut buf).await {
                        Ok(_) => {
                            let message: Message = match bincode::decode_from_slice(
                                &buf,
                                *BINCODE_CONFIG,
                            ) {
                                Ok((message, actual_len)) => {
                                    // Check if the actual length of the message matches the length header
                                    // This is a sanity check to prevent DoS attacks and malformed messages
                                    if actual_len != len {
                                        warn!(
                                            "Peer {} sent a message with length {} bytes, but the actual length is {} bytes. Closing connection.",
                                            peer_addr, len, actual_len
                                        );

                                        // Remove peer from active peers to drop the sender
                                        self.drop_peer(
                                            peer_addr,
                                            Some(format!(
                                                "Peer sent a message with length {} bytes, but the actual length is {} bytes",
                                                len, actual_len
                                            )),
                                        )
                                        .await;

                                        break 'recv;
                                    }

                                    // return the message
                                    message
                                }
                                Err(e) => {
                                    warn!(
                                        "Failed to deserialize peer message: {}. Closing connection. Err: {}",
                                        peer_addr, e
                                    );
                                    trace!(
                                        "Raw contents of message from {}: {:?}",
                                        peer_addr, &buf
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
                        Err(e) => {
                            warn!(
                                "Failed to read data buffer from peer: {}. Closing connection. Err: {}",
                                peer_addr, e
                            );

                            self.drop_peer(
                                peer_addr,
                                format!("Failed to read data buffer from peer: {}", e).into(),
                            )
                            .await;

                            break 'recv;
                        }
                    }
                }
                Err(e) if e.kind() == tokio::io::ErrorKind::UnexpectedEof => {
                    // EOF, connection closed
                    // Check if this was a normal close or a broken pipe

                    self.drop_peer(peer_addr, None).await;

                    break 'recv;
                }
                Err(e) => {
                    warn!(
                        "Failed to read len buffer from peer: {}. Closing connection. Err: {}",
                        peer_addr, e
                    );

                    self.drop_peer(
                        peer_addr,
                        format!("Failed to read len buffer from peer: {}", e).into(),
                    )
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
            Message::ImmediateConnectionClose(disconnect_request) => {
                self.handle_immediate_connection_close(disconnect_request, peer_addr)
                    .await;
            }
            Message::FileOfferRequest(file_offer) => {
                self.handle_file_offer_request(file_offer, peer_addr).await;
            }
            Message::FileOfferResponse(file_offer_response) => {
                self.handle_file_offer_response(file_offer_response, peer_addr)
                    .await;
            }
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
                            // ecdsa_public_key,
                            backend_version,
                        },
                } => {
                    self.backend_event_tx
                        .send(BackendEvent::ConnectionBroken(ConnectionCloseOrBroken {
                            connection_info: ConnectionInfo {
                                name: name.to_string(),
                                ip: peer_addr.ip().to_string(),
                                backend_version: backend_version.to_string(),
                                // identitiy: BASE64_STANDARD.encode(ecdsa_public_key),
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
                                // identitiy: BASE64_STANDARD.encode(&peer_info.ecdsa_public_key),
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
