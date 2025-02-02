use tokio::sync::mpsc;
use tracing::error;

use crate::js_api::{
    backend_event::{BackendEvent, BackendFatal, BackendInfo},
    frontend_event::FrontendEvent,
};

use super::peer_manager::PeerManager;

/// Frontend Manager
///
/// Handles the frontend events, modifying the peer manager accordingly.
pub struct FrontendManager {
    /// Events receiver from the js -> main thread -> tokio
    pub(crate) frontend_event_rx: mpsc::Receiver<FrontendEvent>,
    /// Reference to the peer manager.
    pub(crate) peer_manager: PeerManager,
}

impl FrontendManager {
    pub fn new(
        frontend_event_rx: mpsc::Receiver<FrontendEvent>,
        peer_manager: PeerManager,
    ) -> Self {
        Self {
            frontend_event_rx,
            peer_manager,
        }
    }

    pub async fn shutdown_peer_manager(&mut self) {
        // Shutdown the peer manager gracefully
        self.peer_manager.shutdown().await;

        // // Notify the frontend that the backend has shutdown
        // self.peer_manager
        //     .backend_event_tx
        //     .send(crate::js_api::backend_event::BackendEvent::BackendShutdown)
        //     .await
        //     .expect("Failed to send BackendShutdown event to the frontend");
    }

    pub async fn start_peer_manager(&mut self, bind_addr: String) {
        // Start the peer manager
        let peer_manager = self.peer_manager.clone();
        tokio::spawn(async move {
            match peer_manager.start(bind_addr.as_str()).await.map_err(|e| {
                error!(?e, "Peer Manager failed. Terminating the backend...");
            }) {
                Ok(_) => {
                    // Notify the frontend that the backend has shutdown
                    peer_manager
                        .backend_event_tx
                        .send(BackendEvent::BackendShutdown)
                        .await
                        .expect("Failed to send BackendShutdown event to the frontend");
                }
                Err(_) => {
                    // Ensure we change the state to indicate the backend has failed
                    *peer_manager.shutdown_tx.lock().await = None;

                    // Notify the frontend that the backend has failed
                    peer_manager
                        .backend_event_tx
                        .send(BackendEvent::BackendFatal( BackendFatal {
                            message: "PeerManager failed. Restart the backend. Please check the logs for more information.".to_string(),
                        }))
                        .await
                        .expect("Failed to send BackendShutdown event to the frontend");
                }
            };
        });
    }

    /// Initially start the frontend manager and the peer manager.
    pub async fn start(&mut self, bind_addr: String) {
        // Start the peer manager initially
        self.start_peer_manager(bind_addr).await;
        loop {
            while let Some(event) = self.frontend_event_rx.recv().await {
                // Handle the event
                self.handle_frontend_event(event).await;
            }
        }
    }

    /// Handle the frontend event
    async fn handle_frontend_event(&mut self, event: FrontendEvent) {
        match event {
            FrontendEvent::ConnectRequest(_connect_request) => todo!(),
            FrontendEvent::DisconnectRequest(_disconnect_request) => todo!(),
            FrontendEvent::ConnectionRequestResponse(connection_request_response) => {
                self.handle_connection_request_response(connection_request_response)
                    .await;
            }
            FrontendEvent::TransmitFile(_transmit_file) => todo!(),
            FrontendEvent::FileOfferResponse(_file_offer_response) => todo!(),
            FrontendEvent::CancelFileTransfer(_cancel_file_transfer) => todo!(),
            FrontendEvent::FrontendReady(backend_startup_config) => {
                // We are already beyond the program initialization stage.
                // We are not expecting this event.
                // Complain and ignore the event.
                self.handle_frontend_ready(backend_startup_config).await;
            }
            FrontendEvent::Shutdown => {
                // Shutdown the backend gracefully
                self.shutdown_peer_manager().await;
            }
            FrontendEvent::Start(backend_startup_config) => {
                // If the PeerManager is not running, start it
                // Else, do the same as `FrontendReady` (complain and ignore)

                // Check if the PeerManager is running
                if self.peer_manager.is_running().await {
                    // Complain and ignore
                    self.handle_frontend_ready(backend_startup_config).await;
                } else {
                    // Start the PeerManager
                    self.start_peer_manager(backend_startup_config.bind_addr)
                        .await;

                    // Notify the frontend that the backend has started
                    self.peer_manager
                        .backend_event_tx
                        .send(BackendEvent::BackendReady(BackendInfo {
                            version: env!("CARGO_PKG_VERSION").to_string(),
                        }))
                        .await
                        .expect("Failed to send BackendStarted event to the frontend");
                }
            }
            FrontendEvent::Restart(backend_startup_config) => {
                // Shutdown the PeerManager gracefully
                // Start the PeerManager again
                self.shutdown_peer_manager().await;

                // Wait a bit just to make sure the PeerManager has shutdown.
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;

                // Check if the PeerManager is running
                if self.peer_manager.is_running().await {
                    // Complain and ignore
                    self.handle_frontend_ready(backend_startup_config).await;
                } else {
                    // Start the PeerManager
                    self.start_peer_manager(backend_startup_config.bind_addr)
                        .await;

                    // Notify the frontend that the backend has started
                    self.peer_manager
                        .backend_event_tx
                        .send(BackendEvent::BackendReady(BackendInfo {
                            version: env!("CARGO_PKG_VERSION").to_string(),
                        }))
                        .await
                        .expect("Failed to send BackendStarted event to the frontend");
                }
            }
        }
    }
}
