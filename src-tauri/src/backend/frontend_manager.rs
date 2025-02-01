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

    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            // Receive the next frontend event
            if let Some(event) = self.frontend_event_rx.recv().await {
                // Handle the event
                self.handle_frontend_event(event).await?;
            }
        }
    }

    async fn handle_frontend_event(
        &mut self,
        event: FrontendEvent,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match event {
            FrontendEvent::ConnectRequest(_connect_request) => todo!(),
            FrontendEvent::DisconnectRequest(_disconnect_request) => todo!(),
            FrontendEvent::ConnectionRequestResponse(_connection_request_response) => todo!(),
            FrontendEvent::TransmitFile(_transmit_file) => todo!(),
            FrontendEvent::FileOfferResponse(_file_offer_response) => todo!(),
            FrontendEvent::CancelFileTransfer(_cancel_file_transfer) => todo!(),
            FrontendEvent::FrontendReady => todo!(),
            FrontendEvent::Shutdown => todo!(),
        }
    }
}
