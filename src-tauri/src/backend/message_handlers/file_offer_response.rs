use std::{net::SocketAddr, sync::Arc};

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
                        if let FileTransferDirection::Sending { file_path } =
                            &transfer_state.direction
                        {
                            // We are the one sending the file.
                            // Was the request accepted?
                            if file_offer_response.accept {
                                // Open the file for reading
                                let file_handle = Arc::new(
                                    tokio::fs::File::open(file_path).await.unwrap_or_else(|e| {
                                        // Failed to open the file, update the transfer state to "Error"
                                        transfer_state.status = FileTransferStatus::Error(format!(
                                            "Failed to open file: {}",
                                            e
                                        ));
                                        // TODO: Handle the error properly
                                        panic!("Failed to open file: {}", e);
                                    }),
                                );

                                // Update the transfer state to "InProgress"
                                transfer_state.status = FileTransferStatus::InProgress {
                                    file_handle: file_handle.clone(),
                                };
                            } else {
                                // Update the transfer state to "Rejected"
                                transfer_state.status = FileTransferStatus::Rejected;
                            }
                        } else {
                            // We cannot "accept" a file response if we are the one receiving the file.
                            self.drop_peer(
                                peer_addr,
                                Some("Cannot accept file response while receiving".to_string()),
                            )
                            .await;

                            // Update the transfer state to "Error"
                            transfer_state.status = FileTransferStatus::Error(
                                "Cannot accept file response while receiving".to_string(),
                            );
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
