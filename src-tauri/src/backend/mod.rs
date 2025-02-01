use core::error;

use tokio::sync::mpsc;
use tracing::{error, info};

use crate::js_api::{self, backend_event::BackendMessage};

pub mod peer_manager;
pub mod protocol;

/// Entry point of the backend.
pub async fn init(
    // Events receiver from the js -> main thread -> tokio
    _frontend_event_rx: mpsc::Receiver<js_api::frontend_event::FrontendEvent>,
    // Events sender from tokio -> main thread -> js
    backend_event_tx: mpsc::Sender<js_api::backend_event::BackendEvent>,
) {
    info!("Hello from the backend!");

    // Log versions and other important information

    // Backend Version
    info!("Backend Version:       {}", env!("CARGO_PKG_VERSION"));

    // Build Information via vergen
    info!("Build Information:");
    info!("Rustc Version:         {}", env!("VERGEN_RUSTC_SEMVER"));
    info!("Build Timestamp:       {}", env!("VERGEN_BUILD_TIMESTAMP"));
    info!(
        "Build Target:          {}",
        env!("VERGEN_RUSTC_HOST_TRIPLE")
    );
    info!("Opt Level:             {}", env!("VERGEN_CARGO_OPT_LEVEL"));
    info!("Compile-time Features: {}", env!("VERGEN_CARGO_FEATURES"));

    // Verify mpsc channel communication with the frontend is working
    // by sending a BackendReady event to the frontend.
    // If this fails, we should error and terminate the backend.
    backend_event_tx
        .send(js_api::backend_event::BackendEvent::BackendReady)
        .await
        .map_err(|e| {
            error!(?e, "Failed to send BackendReady event to the frontend");
            error!("Indicates mpsc channel failure, we cannot communicate with the frontend.");
            error!("Therefore, we are terminating here...");
            e
        })
        .expect("Failed to send BackendReady event to the frontend");

    // Create a new PeerManager
    let peer_manager = peer_manager::PeerManager::new(backend_event_tx.clone());

    // Start the PeerManager
    peer_manager
        .start("0.0.0.0:8080")
        .await
        .map_err(|e| {
            error!(?e, "PeerManager panicked, terminating backend");
            e
        })
        .unwrap();

    // loop {
    //     backend_event_tx
    //         .send(js_api::backend_event::BackendEvent::Message(
    //             BackendMessage {
    //                 message: "Test".to_string(),
    //             },
    //         ))
    //         .await
    //         .unwrap();
    //     tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    // }
}
