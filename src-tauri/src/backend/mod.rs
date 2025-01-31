use tokio::sync::mpsc;
use tracing::info;

use crate::js_api;

pub mod protocol;

/// Entry point of the backend.
pub async fn init(
    // Events receiver from the js -> main thread -> tokio
    _frontend_event_rx: mpsc::Receiver<js_api::frontend_event::FrontendEvent>,
    // Events sender from tokio -> main thread -> js
    backend_event_tx: mpsc::Sender<js_api::backend_event::BackendEvent>,
) {
    info!("Hello from the backend!");

    // Backend event: BackendReady
    backend_event_tx
        .send(js_api::backend_event::BackendEvent::BackendReady)
        .await
        .unwrap();

    loop {
        // hlt
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}
