//! # Protocol
//!
//! This module defines the protocol for peers to communicate with each other.
//!
//! ## Spec
//!
//! Consists of:
//!
//! - Main header (4 bytes): Packet Body Length (in bytes) (Big Endian)
//! - Message body: The message itself, encoded using bincode v2 (Little Endian, variable length integers) (See [BINCODE_CONFIG])
//!
//! Maximum message size is 10 MB (10 * 1024 * 1024 bytes) (see [MAX_MESSAGE_SIZE])
//!
//! If the message size exceeds this limit, the connection will be closed immediately.

use bincode::config::{self, Configuration};
use once_cell::sync::Lazy;
use uuid::Uuid;

use super::peer_manager::PeerInfo;

/// Bincode v2 Configuration static
pub static BINCODE_CONFIG: Lazy<Configuration> = Lazy::new(|| {
    config::standard()
        .with_little_endian()
        .with_variable_int_encoding()
});

/// Maximum message size for the protocol in bytes (10 MB)
pub const MAX_MESSAGE_SIZE: usize = 1024 * 1024 * 10; // 10 MB

/// The Message Enum.
///
/// This is the protocol for the backend to communicate with the frontend.
#[derive(Debug, bincode::Encode, bincode::Decode)]
pub enum Message {
    /// Keep-alive message to prevent TCP connections from timing out
    KeepAlive,
    /// Request to connect to the peer
    ConnectRequest(ConnectionInfo),
    /// Response to a connect request
    ConnectResponse(ConnectionResponse),
    /// Request to disconnect from the peer
    DisconnectRequest(DisconnectRequest),
    /// Response to a disconnect request
    DisconnectAck,
    /// Immediate connection close message to the peer. Not to be ACKed. Usually sent when peer is shutting down.
    ImmediateConnectionClose(DisconnectRequest),
    /// Request to send a message to the peer
    FileOfferRequest(FileOffer),
    /// Response to a file offer request
    FileOfferResponse(FileOfferResponse),
    /// Request to send a chunk of a file to the peer
    FileChunk(FileChunk),
    /// Response to a file chunk request
    FileChunkAck(FileChunkAck),
    /// Request to send a file done message to the peer
    FileDone(FileDone),
    /// Response to a file done request
    FileDoneResult(FileDoneResult),
}

#[derive(Debug, bincode::Encode, bincode::Decode)]
pub struct EcdsaConnectionInfo {
    pub public_key: Vec<u8>,
    pub signature: Vec<u8>,
    pub nonce: Vec<u8>,
}

#[derive(Debug, bincode::Encode, bincode::Decode)]
pub struct ConnectionInfo {
    pub name: String,
    // Use Cargo.toml to set the version
    pub backend_version: String,
    // /// The ECDSA public key of the peer
    // pub identitiy: EcdsaConnectionInfo,
}

impl From<ConnectionInfo> for PeerInfo {
    fn from(info: ConnectionInfo) -> Self {
        PeerInfo {
            name: info.name,
            backend_version: info.backend_version,
            // ecdsa_public_key: info.identitiy,
        }
    }
}

#[derive(Debug, bincode::Encode, bincode::Decode)]
pub struct ConnectionResponse {
    pub permit: ConnectionPermit,
    pub message: Option<String>,
}

#[derive(Debug, bincode::Encode, bincode::Decode)]
pub struct DisconnectRequest {
    pub message: Option<String>,
}

#[derive(Debug, bincode::Encode, bincode::Decode)]
pub enum ConnectionPermit {
    Permit {
        /// The ConnectionInfo of the peer
        identitiy: ConnectionInfo,
    },
    Deny,
}

#[derive(Debug, bincode::Encode, bincode::Decode)]
pub struct FileOffer {
    pub filename: String,
    #[bincode(with_serde)]
    pub unique_id: Uuid,
    pub size: u64,
    pub chunk_len: u64,
}

#[derive(Debug, bincode::Encode, bincode::Decode)]
pub struct FileOfferResponse {
    #[bincode(with_serde)]
    pub unique_id: Uuid,
    pub accept: bool,
}

#[derive(Debug, bincode::Encode, bincode::Decode)]
pub struct FileChunk {
    #[bincode(with_serde)]
    pub unique_id: Uuid,
    pub chunk_id: u64,
    pub chunk_len: u64,
    pub data: Vec<u8>,
}

#[derive(Debug, bincode::Encode, bincode::Decode)]
pub struct FileChunkAck {
    #[bincode(with_serde)]
    pub unique_id: Uuid,
    pub chunk_id: u64,
}

#[derive(Debug, bincode::Encode, bincode::Decode)]
pub struct FileDone {
    #[bincode(with_serde)]
    pub unique_id: Uuid,
    pub checksum: Vec<u8>,
}

#[derive(Debug, bincode::Encode, bincode::Decode)]
pub struct FileDoneResult {
    #[bincode(with_serde)]
    pub unique_id: Uuid,
    pub success: bool,
    pub message: Option<String>,
}
