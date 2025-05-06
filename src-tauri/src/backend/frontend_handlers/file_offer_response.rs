use uuid::Uuid;

use crate::{
    backend::{
        frontend_manager::FrontendManager,
        peer_manager::{FileTransferDirection, FileTransferStatus},
        protocol::{self, Message},
    },
    js_api::{
        backend_event::{BackendEvent, BadFrontendEvent},
        frontend_event::{FileOfferResponse, FrontendEvent},
    },
};

impl FrontendManager {
    pub(crate) async fn handle_file_offer_response(
        &mut self,
        file_offer_response: FileOfferResponse,
    ) {
        // Check if the file transfer ID exists in the peer manager's transfer state
        // If it does, update the transfer state with the new status
        // If it doesn't, complain to the frontend that the transfer ID is invalid

        let unique_id: Uuid = match file_offer_response.unique_id.parse() {
            Ok(unique_id) => unique_id,
            Err(e) => {
                // Invalid UUID
                self.peer_manager
                    .backend_event_tx
                    .send(BackendEvent::BadFrontendEvent(BadFrontendEvent {
                        event: FrontendEvent::FileOfferResponse(file_offer_response),
                        error: format!("Invalid File Transfer ID (UUID): {}", e),
                    }))
                    .await
                    .expect("Failed to send BadFrontendEvent event to the backend");
                return;
            }
        };

        let mut active_transfers = self.peer_manager.active_transfers.lock().await;

        if let Some(transfer) = active_transfers.get_mut(&unique_id) {
            // We cannot "accept" a file offer if we are the one sending the file.
            if transfer.direction == FileTransferDirection::Sending {
                self.peer_manager
                    .backend_event_tx
                    .send(BackendEvent::BadFrontendEvent(BadFrontendEvent {
                        event: FrontendEvent::FileOfferResponse(file_offer_response),
                        error: "Cannot accept a file offer when sending a file.".to_string(),
                    }))
                    .await
                    .expect("Failed to send BadFrontendEvent event to the backend");
                return;
            }

            // Send the file offer response to the peer
            if let Some(peer) = self
                .peer_manager
                .active_peers
                .lock()
                .await
                .get_mut(&transfer.peer_addr)
            {
                peer.tx
                    .send(Message::FileOfferResponse(protocol::FileOfferResponse {
                        unique_id: transfer.unique_id,
                        accept: file_offer_response.accept,
                    }))
                    .await
                    .expect("Failed to send FileOfferResponse message to the peer");

                if file_offer_response.accept {
                    // Accepted!
                    // We can accept file chunks from the peer now!
                    transfer.status = FileTransferStatus::InProgress;
                } else {
                    // Rejected.
                    // Remove the transfer state from the active transfers
                    active_transfers.remove(&unique_id);
                }
            } else {
                // Peer is not connected, remove the transfer state
                active_transfers.remove(&unique_id);
                // Notify frontend of error
                self.peer_manager
                    .backend_event_tx
                    .send(BackendEvent::BadFrontendEvent(BadFrontendEvent {
                        event: FrontendEvent::FileOfferResponse(file_offer_response),
                        error: "Peer is not connected".to_string(),
                    }))
                    .await
                    .expect("Failed to send BadFrontendEvent event to the backend");
            }
        } else {
            // Invalid file transfer ID
            self.peer_manager
                .backend_event_tx
                .send(BackendEvent::BadFrontendEvent(BadFrontendEvent {
                    event: FrontendEvent::FileOfferResponse(file_offer_response),
                    error: "Invalid file transfer ID: ID does not exist.".to_string(),
                }))
                .await
                .expect("Failed to send BadFrontendEvent event to the backend");
        }
    }
}
