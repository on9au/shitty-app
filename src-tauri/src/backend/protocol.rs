use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Message {
    /// Failed to parse a message, disconnect the peer
    InvalidMessage(String),
    /// Request to connect to the peer
    ConnectRequest,
    /// Response to a connect request
    ConnectResponse {
        success: bool,
        message: Option<String>,
    },
    /// Request to disconnect from the peer
    DisconnectRequest(String),
    /// Response to a disconnect request
    DisconnectAck,
    /// Request to send a message to the peer
    FileOfferRequest(FileOffer),
    /// Response to a file offer request
    FileOfferResponse { accept: bool },
    /// Request to send a chunk of a file to the peer
    FileChunk(FileChunk),
    /// Response to a file chunk request
    FileChunkAck(FileChunkAck),
    /// Request to send a file done message to the peer
    FileDone(FileDone),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileOffer {
    pub filename: String,
    pub unique_id: u64,
    pub size: u64,
    pub chunk_len: u64,
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
