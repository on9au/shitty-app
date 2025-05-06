use std::net::SocketAddr;

use crate::backend::{
    peer_manager::{FileTransferDirection, FileTransferStatus, PeerManager, PeerState},
    protocol::FileOfferResponse,
};

impl PeerManager {
    pub async fn handle_file_offer_response(
        &self,
        file_offer_response: FileOfferResponse,
        peer_addr: SocketAddr,
    ) {
        // We got a file offer response from a peer.
        // Check if the peer is connected
        // If the peer is connected, update the file transfer state in the PeerManager
        // If the peer is not connected, ignore the response

        let mut peers = self.active_peers.lock().await;

        if let Some(peer) = peers.get_mut(&peer_addr) {
            match &peer.state {
                PeerState::Connected { .. } => {
                    // Peer is not authenticated yet, but they sent a file offer response?
                    // Disconnect the peer
                    self.drop_peer(
                        peer_addr,
                        Some("Peer sent a file offer response before authentication".to_string()),
                    )
                    .await;
                }
                PeerState::Authenticated { .. } => {
                    // Peer is authenticated.
                    // Update the file transfer state in the PeerManager
                    if let Some(transfer_state) = self
                        .active_transfers
                        .lock()
                        .await
                        .get_mut(&file_offer_response.unique_id)
                    {
                        // We cannot "accept" a file response if we are the one receiving the file.
                        if transfer_state.direction == FileTransferDirection::Receiving {
                            self.drop_peer(
                                peer_addr,
                                Some("Cannot accept file response while receiving".to_string()),
                            )
                            .await;

                            // Update the transfer state to "Error"
                            transfer_state.status = FileTransferStatus::Error(
                                "Cannot accept file response while receiving".to_string(),
                            );

                            return;
                        }

                        // Was the request accepted?
                        if file_offer_response.accept {
                            // Update the transfer state to "InProgress"
                            transfer_state.status = FileTransferStatus::InProgress;

                            // TODO: Begin the file transfer
                        } else {
                            // Update the transfer state to "Rejected"
                            transfer_state.status = FileTransferStatus::Rejected;
                        }
                    }
                }
                PeerState::Disconnecting { .. } => {
                    // Peer is already disconnecting, but they sent a file offer response?
                    // Disconnect the peer
                    self.drop_peer(peer_addr, None).await;
                }
            }
        }
    }
}
