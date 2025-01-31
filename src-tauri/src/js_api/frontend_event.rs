use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::js_rs_interop::FrontendEventTx;

/// Enum of events that occur in the frontend and should be sent to the backend.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum FrontendEvent {
    /// Transmit a file to the user.
    TransmitFile(String),
    /// Accept or reject a file offer.
    FileOfferResponse(FileOfferResponse),
    /// Cancel a file transfer.
    CancelFileTransfer(CancelFileTransfer),
}

/// Struct representing a file offer response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileOfferResponse {
    /// The unique identifier of the file being offered.
    pub unique_id: u64,
    /// Whether the file offer is accepted.
    pub accept: bool,
}

/// Struct representing a file transfer cancellation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelFileTransfer {
    /// The unique identifier of the file transfer to cancel.
    pub unique_id: u64,
    /// Optional message to send with the cancellation.
    pub message: Option<String>,
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
