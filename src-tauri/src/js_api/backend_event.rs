use serde::{Deserialize, Serialize};

/// Enum of events that occur in the backend and should be sent to the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackendEvent {
    /// Backend error.
    BackendError(String),
    /// Backend fatal/panic error. The backend will shut down after sending this.
    BackendFatal(String),
    /// Backend is ready to receive messages.
    BackendReady,
    /// Backend is shutting down gracefully.
    BackendShutdown,
    /// Backend warning.
    BackendWarning(String),

    /// A file offer from the backend to the frontend.
    FileOffer(FileOffer),
    /// A file transfer completion from the backend to the frontend.
    FileTransferComplete(FileTransferComplete),
    /// A file transfer error from the backend to the frontend.
    FileTransferError(FileTransferError),
    /// A file transfer progress update from the backend to the frontend.
    FileTransferProgress(FileTransferProgress),
    /// A general message from the backend to the frontend.
    Message(String),
}

/// Struct representing a file offer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileOffer {
    /// The filename of the file being offered.
    pub filename: String,
    /// A unique identifier for the file.
    pub unique_id: u64,
    /// The size of the file in bytes.
    pub size: u64,
}

/// Struct representing a file transfer completion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileTransferComplete {
    /// The unique identifier of the file that was transferred.
    pub unique_id: u64,
}

/// Struct representing a file transfer error.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileTransferError {
    /// The unique identifier of the file that had an error.
    pub unique_id: u64,
    /// The error message.
    pub message: String,
}

/// Struct representing a file transfer progress update.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileTransferProgress {
    /// The unique identifier of the file being transferred.
    pub unique_id: u64,
    /// The number of bytes transferred so far.
    pub bytes_transferred: u64,
    /// The total number of bytes to transfer.
    pub total_bytes: u64,
}
