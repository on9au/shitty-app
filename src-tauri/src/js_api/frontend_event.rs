use serde::{Deserialize, Serialize};
use tracing::debug;
use ts_rs::TS;

use std::ops::{Deref, DerefMut};

/// Enum of events that occur in the frontend and should be sent to the backend.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(tag = "type")]
#[ts(export)]
pub enum FrontendEvent {
    /// New request: Request to connect to a peer.
    ConnectRequest(ConnectRequest),
    /// New request: Request to disconnect from an already connected peer.
    DisconnectRequest(DisconnectRequest),

    /// Response: Response to a receiving connection request.
    ConnectionRequestResponse(ConnectionRequestResponse),

    /// New request: Transmit a file to the user.
    TransmitFile(TransmitFile),
    /// Response: Accept or reject a file offer.
    FileOfferResponse(FileOfferResponse),
    /// New request: Cancel a file transfer.
    CancelFileTransfer(CancelFileTransfer),

    /// Startup: Frontend is ready to receive messages from the backend.
    FrontendReady(BackendStartupConfig),
    /// Shutdown: Shutdown the backend gracefully.
    Shutdown,
}

/// Struct representing a connection request.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ConnectRequest {
    /// The IP address of the peer to connect to.
    pub ip: String,
}

/// Struct representing a disconnection request.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct DisconnectRequest {
    /// The IP address of the peer to disconnect from.
    pub ip: String,
    /// Optional message to send with the disconnection.
    pub message: Option<String>,
}

/// Struct representing a connection request response.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ConnectionRequestResponse {
    /// The IP address of the peer to connect to.
    pub ip: String,
    /// Whether the connection request is accepted.
    pub accept: bool,
    /// Optional message to send with the connection request response if rejected.
    /// Ignored if accept is true.
    pub message: Option<String>,
}

/// Struct representing a file transmission request.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TransmitFile {
    /// The absolute path to the file to transmit.
    pub path: String,
    /// The filename to transmit.
    pub filename: String,
}

/// Struct representing a file offer response.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct FileOfferResponse {
    /// The unique identifier of the file being offered.
    pub unique_id: u64,
    /// Whether the file offer is accepted.
    pub accept: bool,
}

/// Struct representing a file transfer cancellation.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct CancelFileTransfer {
    /// The unique identifier of the file transfer to cancel.
    pub unique_id: u64,
    /// Optional message to send with the cancellation.
    pub message: Option<String>,
}

/// Struct representing the configuration for the backend startup.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct BackendStartupConfig {
    /// Socket address to bind to. (e.g. "0.0.0.0:8080 or [::1]:8080")
    pub bind_addr: String,
}

/// Async Process Input Transmitter State
///
/// Main Thread -> Tokio
pub struct FrontendEventTx {
    inner: tokio::sync::Mutex<tokio::sync::mpsc::Sender<FrontendEvent>>,
}

impl FrontendEventTx {
    pub fn new(tx: tokio::sync::mpsc::Sender<FrontendEvent>) -> Self {
        Self {
            inner: tokio::sync::Mutex::new(tx),
        }
    }
}

impl Deref for FrontendEventTx {
    type Target = tokio::sync::Mutex<tokio::sync::mpsc::Sender<FrontendEvent>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for FrontendEventTx {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

/// Tauri JS API for pushing frontend events to the backend.
///
/// - The event emitted is `frontend_event` (handled by main thread sender).
#[tauri::command]
pub async fn push_frontend_event(
    event: FrontendEvent,
    state: tauri::State<'_, FrontendEventTx>,
) -> Result<(), String> {
    // Log the event
    debug!(?event, "Frontend Event Received");
    // Send the event to the backend mpsc channel
    // (Frontend Events) Js -> Main Thread -> Tokio
    let async_process_input_tx = state.lock().await;
    async_process_input_tx
        .send(event)
        .await
        .map_err(|e| e.to_string())
}
