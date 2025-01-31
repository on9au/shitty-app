use tracing::info;

use crate::js_api;

pub mod protocol;

/// Entry point of the backend.
pub async fn init(
    // Events receiver from the js -> main thread -> tokio
    _frontend_event_rx: tokio::sync::mpsc::Receiver<js_api::frontend_event::FrontendEvent>,
    // Events sender from tokio -> main thread -> js
    _backend_event_tx: tokio::sync::mpsc::Sender<js_api::backend_event::BackendEvent>,
) {
    info!("Hello from the backend!");
    loop {
        // hlt
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}
