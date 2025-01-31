use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Enum of events that occur in the backend and should be sent to the frontend.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(tag = "type")]
#[ts(export)]
pub enum BackendEvent {
    /// Backend error.
    BackendError(BackendError),
    /// Backend fatal/panic error. The backend will shut down after sending this.
    BackendFatal(BackendFatal),
    /// Backend is ready to receive messages.
    /// If this is not sent, the frontend should assume the backend is not ready,
    /// or should assume the backend or mpsc channel is failing (which will therefore fail the backend).
    BackendReady,
    /// Backend is shutting down gracefully.
    BackendShutdown,
    /// Backend warning.
    BackendWarning(BackendWarning),

    /// A connection request received from a peer.
    ConnectRequest(ConnectionInfo),
    /// An automatic connection closure due to an error.
    /// For example, invalid version, blacklisted IP/name, etc.
    AutoConnectionClose(ConnectionInfo),
    /// A connection closure with a peer. Can be used as an acknowledgement of a disconnect request.
    ConnectionClose(ConnectionCloseOrBroken),
    /// An unexpected connection closure with a peer. Unlike ConnectionClose, this is due to an error.
    ConnectionBroken(ConnectionCloseOrBroken),

    /// A file offer from the backend to the frontend.
    FileOffer(FileOffer),
    /// A file transfer completion from the backend to the frontend.
    FileTransferComplete(FileTransferComplete),
    /// A file transfer error from the backend to the frontend.
    FileTransferError(FileTransferError),
    /// A file transfer progress update from the backend to the frontend.
    FileTransferProgress(FileTransferProgress),
    /// A general message from the backend to the frontend.
    Message(BackendMessage),
}

/// Struct representing a backend error.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct BackendError {
    /// The error message.
    pub message: String,
}

/// Struct representing a backend fatal/panic.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct BackendFatal {
    /// The error message.
    pub message: String,
}

/// Struct representing a backend warning.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct BackendWarning {
    /// The warning message.
    pub message: String,
}

/// Struct representing a connection info.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ConnectionInfo {
    /// The name of the connection.
    pub name: String,
    /// The IP address of the connection.
    pub ip: String,
    /// The version of the backend.
    pub backend_version: String,
}

/// Struct representing an unexpected connection closure.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ConnectionCloseOrBroken {
    /// The connection info.
    pub connection_info: ConnectionInfo,
    /// The error message.
    pub message: Option<String>,
}

/// Struct representing a file offer.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct FileOffer {
    /// The filename of the file being offered.
    pub filename: String,
    /// A unique identifier for the file.
    pub unique_id: u64,
    /// The size of the file in bytes.
    pub size: u64,
}

/// Struct representing a file transfer completion.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct FileTransferComplete {
    /// The unique identifier of the file that was transferred.
    pub unique_id: u64,
}

/// Struct representing a file transfer error.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct FileTransferError {
    /// The unique identifier of the file that had an error.
    pub unique_id: u64,
    /// The error message.
    pub message: String,
}

/// Struct representing a file transfer progress update.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct FileTransferProgress {
    /// The unique identifier of the file being transferred.
    pub unique_id: u64,
    /// The number of bytes transferred so far.
    pub bytes_transferred: u64,
    /// The total number of bytes to transfer.
    pub total_bytes: u64,
    /// Sending or receiving the file?
    pub sending: FileTransferDirection,
}

/// Enum representing whether a file transfer is sending or receiving.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum FileTransferDirection {
    /// Sending the file.
    Sending,
    /// Receiving the file.
    Receiving,
}

/// Struct representing a general message from the backend.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct BackendMessage {
    /// The message.
    pub message: String,
}
