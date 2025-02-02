use tokio::sync::mpsc;

use crate::js_api::frontend_event::FrontendEvent;

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

    pub async fn shutdown(&mut self) {
        // Shutdown the peer manager gracefully
        self.peer_manager.shutdown().await;

        // Notify the frontend that the backend has shutdown
        self.peer_manager
            .backend_event_tx
            .send(crate::js_api::backend_event::BackendEvent::BackendShutdown)
            .await
            .expect("Failed to send BackendShutdown event to the frontend");
    }

    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            while let Some(event) = self.frontend_event_rx.recv().await {
                // Handle the event
                self.handle_frontend_event(event).await;
            }
        }
    }

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
                self.handle_frontend_ready(backend_startup_config).await;
            }
            FrontendEvent::Shutdown => {
                // Shutdown the backend gracefully
                self.shutdown().await;
            }
        }
    }
}
