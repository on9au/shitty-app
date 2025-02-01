use tokio::sync::mpsc;
use tracing::{error, info};

use crate::js_api::{self, backend_event::BackendFatal};

pub mod peer_manager;
pub mod protocol;

/// Entry point of the backend.
pub async fn init(
    // Events receiver from the js -> main thread -> tokio
    mut frontend_event_rx: mpsc::Receiver<js_api::frontend_event::FrontendEvent>,
    // Events sender from tokio -> main thread -> js
    backend_event_tx: mpsc::Sender<js_api::backend_event::BackendEvent>,
) {
    info!("Hello from the backend!");

    // Log versions and other important information

    // Backend Version
    info!("Backend Version:         {}", env!("CARGO_PKG_VERSION"));

    // Certificate Information
    info!("Certificate Information: {}", "TBC");

    // Build Information via vergen
    info!("Build Information:");
    info!(" Rustc Version:          {}", env!("VERGEN_RUSTC_SEMVER"));
    info!(" Rustc Channel:          {}", env!("VERGEN_RUSTC_CHANNEL"));
    info!(
        " Build Timestamp:        {}",
        env!("VERGEN_BUILD_TIMESTAMP")
    );
    info!(
        " Target Triple:          {}",
        env!("VERGEN_CARGO_TARGET_TRIPLE")
    );
    info!(
        " Build Target:           {}",
        env!("VERGEN_RUSTC_HOST_TRIPLE")
    );
    info!(
        " Opt Level:              {}",
        env!("VERGEN_CARGO_OPT_LEVEL")
    );
    info!(" Debug:                  {}", env!("VERGEN_CARGO_DEBUG"));
    info!(" Compile-time Features:  {}", env!("VERGEN_CARGO_FEATURES"));

    // Awaiting confirmation from the frontend that it is ready
    // to receive messages from the backend.
    info!("Awaiting confirmation from the frontend...");
    match frontend_event_rx.recv().await {
        Some(js_api::frontend_event::FrontendEvent::FrontendReady) => {
            info!("Frontend is ready to receive messages from the backend.");
        }
        Some(other_event) => {
            let error_msg = format!(
                "Unexpected event received from the frontend. Expected: {:?}, but got: {:?}",
                js_api::frontend_event::FrontendEvent::FrontendReady,
                other_event
            );
            error!(error_msg);
            error!("Terminating backend...");

            backend_event_tx
                .send(js_api::backend_event::BackendEvent::BackendFatal(
                    BackendFatal { message: error_msg },
                ))
                .await
                .expect("Failed to send BackendFatal event to the frontend");

            return;
        }
        None => {
            let error_msg = "Frontend event receiver closed unexpectedly.";

            error!("{}", error_msg);

            backend_event_tx
                .send(js_api::backend_event::BackendEvent::BackendFatal(
                    BackendFatal {
                        message: error_msg.to_string(),
                    },
                ))
                .await
                .expect("Failed to send BackendFatal event to the frontend");

            error!("Terminating backend...");
            return;
        }
    }

    // Verify mpsc channel communication with the frontend is working
    // by sending a BackendReady event to the frontend.
    // If this fails, we should error and terminate the backend.
    backend_event_tx
        .send(js_api::backend_event::BackendEvent::BackendReady(
            js_api::backend_event::BackendInfo {
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
        ))
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
