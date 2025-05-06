use std::net::SocketAddr;

use tokio::fs::File;
use tracing::warn;
use uuid::Uuid;

use crate::{
    backend::{
        frontend_manager::FrontendManager,
        peer_manager::{FileTransferDirection, FileTransferStatus},
        protocol::{FileOffer, Message},
    },
    js_api::{
        backend_event::{BackendEvent, BadFrontendEvent},
        frontend_event::{FrontendEvent, TransmitFile},
    },
};

impl FrontendManager {
    pub(crate) async fn handle_transmit_file(&mut self, transmit_file: TransmitFile) {
        // Initiate the file transfer process with the peer

        // Parse the IP address
        let peer_addr: SocketAddr = match transmit_file.ip.parse() {
            Ok(peer_addr) => peer_addr,
            Err(_) => {
                // Invalid IP address
                self.peer_manager
                    .backend_event_tx
                    .send(BackendEvent::BadFrontendEvent(BadFrontendEvent {
                        event: FrontendEvent::TransmitFile(transmit_file),
                        error: "Invalid IP address".to_string(),
                    }))
                    .await
                    .expect("Failed to send BadFrontendEvent event to the backend");
                return;
            }
        };

        // Open the file
        let file = match File::open(&transmit_file.path).await {
            Ok(f) => f,
            Err(e) => {
                // Notify frontend of error
                self.peer_manager
                    .backend_event_tx
                    .send(BackendEvent::BadFrontendEvent(BadFrontendEvent {
                        event: FrontendEvent::TransmitFile(transmit_file.clone()),
                        error: format!("Failed to open file: {}", e),
                    }))
                    .await
                    .expect("Failed to send BadFrontendEvent");
                return;
            }
        };
        let metadata = match file.metadata().await {
            Ok(m) => m,
            Err(e) => {
                // Notify frontend of error
                self.peer_manager
                    .backend_event_tx
                    .send(BackendEvent::BadFrontendEvent(BadFrontendEvent {
                        event: FrontendEvent::TransmitFile(transmit_file.clone()),
                        error: format!("Failed to get file metadata: {}", e),
                    }))
                    .await
                    .expect("Failed to send BadFrontendEvent");
                return;
            }
        };
        let size = metadata.len();
        let chunk_len = 1024 * 1024; // 1 MB for now
        let unique_id = Uuid::new_v4();

        let mut peers = self.peer_manager.active_peers.lock().await;

        if let Some(peer) = peers.get_mut(&peer_addr) {
            // Peer is connected
            // Send a `TransmitFile` message to the peer
            // Send FileOfferRequest
            let offer = FileOffer {
                filename: transmit_file.filename.clone(),
                unique_id,
                size,
                chunk_len,
            };

            match peer.tx.send(Message::FileOfferRequest(offer)).await {
                Ok(_) => {
                    // Message sent successfully
                    // Store transfer state
                    self.peer_manager.active_transfers.lock().await.insert(
                        unique_id,
                        crate::backend::peer_manager::FileTransferState {
                            unique_id,
                            peer_addr,
                            direction: FileTransferDirection::Sending,
                            filename: transmit_file.filename,
                            total_size: size,
                            bytes_transferred: 0,
                            chunk_len,
                            status: FileTransferStatus::InProgress,
                        },
                    );
                }
                Err(e) => {
                    // Failed to send the message
                    // Disconnect the peer except override the message with the error
                    warn!(
                        ?e,
                        "Failed to send TransmitFile message to the peer. Disconnecting the peer with an error message."
                    );
                    self.peer_manager
                        .drop_peer(
                            peer_addr,
                            Some("Failed to send TransmitFile message to the peer".to_string()),
                        )
                        .await;
                }
            }
        } else {
            // Peer is not connected
            // Ignore the request
            warn!(
                "Tried to TransmitFile to a peer that is not connected: {}",
                peer_addr
            );

            // Complain to the frontend
            self.peer_manager
                .backend_event_tx
                .send(BackendEvent::BadFrontendEvent(BadFrontendEvent {
                    event: FrontendEvent::TransmitFile(transmit_file),
                    error: format!("Peer {} is not connected", peer_addr),
                }))
                .await
                .expect("Failed to send BadFrontendEvent event to the backend");
        }
    }
}
