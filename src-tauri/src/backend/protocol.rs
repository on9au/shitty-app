use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
pub struct EcdsaConnectionInfo {
    pub public_key: Vec<u8>,
    pub signature: Vec<u8>,
    pub nonce: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConnectionInfo {
    pub name: String,
    // Use Cargo.toml to set the version
    pub backend_version: String,
    /// The ECDSA public key of the peer
    pub identitiy: EcdsaConnectionInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConnectionResponse {
    pub permit: ConnectionPermit,
    pub message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DisconnectRequest {
    pub message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ConnectionPermit {
    Permit {
        /// The ECDSA public key of the peer
        identitiy: EcdsaConnectionInfo,
    },
    Deny,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileOffer {
    pub filename: String,
    pub unique_id: u64,
    pub size: u64,
    pub chunk_len: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileOfferResponse {
    pub unique_id: u64,
    pub accept: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileChunk {
    pub unique_id: u64,
    pub chunk_id: u64,
    pub chunk_len: u64,
    pub data: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileChunkAck {
    pub unique_id: u64,
    pub chunk_id: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileDone {
    pub unique_id: u64,
    pub checksum: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileDoneResult {
    pub unique_id: u64,
    pub success: bool,
    pub message: Option<String>,
}
