use tokio::sync::mpsc;
use tracing::{error, info};

use crate::js_api::{self, backend_event::BackendFatal};

pub mod frontend_handlers;
pub mod frontend_manager;
pub mod message_handlers;
pub mod peer_manager;
pub mod protocol;

/// Log versions and other important information.
/// This macro is used to log the versions of the backend and frontend.
macro_rules! log_backend_info {
    () => {
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
    };
}

/// Awaits confirmation from the frontend that it is ready to receive messages from the backend.
/// If the frontend is not ready, the backend should terminate.
async fn await_frontend_ready(
    frontend_event_rx: &mut mpsc::Receiver<js_api::frontend_event::FrontendEvent>,
    backend_event_tx: &mpsc::Sender<js_api::backend_event::BackendEvent>,
) -> bool {
    info!("Awaiting confirmation from the frontend...");
    match frontend_event_rx.recv().await {
        Some(js_api::frontend_event::FrontendEvent::FrontendReady) => {
            info!("Frontend is ready to receive messages from the backend.");
            true
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

            false
        }
        None => {
            let error_msg = "Frontend event receiver closed unexpectedly.";

            error!("{}", error_msg);

            error!("Terminating backend...");

            false
        }
    }
}

/// Verify mpsc channel communication with the frontend is working by sending a BackendReady event to the frontend.
/// If this fails, we should error and terminate the backend.
async fn verify_mpsc_channel(
    backend_event_tx: &mpsc::Sender<js_api::backend_event::BackendEvent>,
) -> bool {
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
        .is_ok()
}

/// Entry point of the backend.
pub async fn init(
    // Events receiver from the js -> main thread -> tokio
    mut frontend_event_rx: mpsc::Receiver<js_api::frontend_event::FrontendEvent>,
    // Events sender from tokio -> main thread -> js
    backend_event_tx: mpsc::Sender<js_api::backend_event::BackendEvent>,
) {
    info!("Hello from the backend!");

    // Log versions and other important information

    log_backend_info!();

    // Awaiting confirmation from the frontend that it is ready
    // to receive messages from the backend.
    if !await_frontend_ready(&mut frontend_event_rx, &backend_event_tx).await {
        return;
    }

    // Verify mpsc channel communication with the frontend is working
    if !verify_mpsc_channel(&backend_event_tx).await {
        return;
    }

    // Create a new PeerManager
    let peer_manager = peer_manager::PeerManager::new(backend_event_tx.clone());

    // Create a new FrontendManager
    let mut frontend_manager =
        frontend_manager::FrontendManager::new(frontend_event_rx, peer_manager.clone());

    // Start the PeerManager
    let peer_manager_thread = tokio::spawn(async move {
        peer_manager
            .start("0.0.0.0:8080")
            .await
            .map_err(|e| {
                error!(?e, "PeerManager panicked, terminating backend");
                e
            })
            .unwrap();
    });

    // Start the FrontendManager
    let frontend_manager_thread = tokio::spawn(async move {
        frontend_manager
            .start()
            .await
            .map_err(|e| {
                error!(?e, "FrontendManager panicked, terminating backend");
                e
            })
            .unwrap();
    });

    // If any of the threads panic or return an error, we should terminate the backend.
    tokio::select! {
        _ = peer_manager_thread => {
            error!("PeerManager thread terminated unexpectedly, terminating backend...");
        }
        _ = frontend_manager_thread => {
            error!("FrontendManager thread terminated unexpectedly, terminating backend...");
        }
    }
}
