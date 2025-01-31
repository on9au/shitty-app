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
    BackendReady,
    /// Backend is shutting down gracefully.
    BackendShutdown,
    /// Backend warning.
    BackendWarning(BackendWarning),

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
