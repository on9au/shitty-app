use std::net::SocketAddr;

use crate::{
    backend::{
        peer_manager::{
            FileTransferDirection, FileTransferState, FileTransferStatus, PeerManager, PeerState,
        },
        protocol,
    },
    js_api::backend_event::{BackendEvent, FileOffer},
};

impl PeerManager {
    pub async fn handle_file_offer_request(
        &self,
        file_offer: protocol::FileOffer,
        peer_addr: SocketAddr,
    ) {
        // We got a file offer request from a peer.
        // Check if the peer is connected
        // If the peer is connected, add the file transfer state to the PeerManager
        // Send a backend event to the frontend with the file offer request
        // If the peer is not connected, ignore the request

        let mut peers = self.active_peers.lock().await;

        if let Some(peer) = peers.get_mut(&peer_addr) {
            match &peer.state {
                PeerState::Connected { .. } => {
                    // Peer is not authenticated yet, but they sent a file offer request?
                    // Disconnect the peer
                    self.drop_peer(
                        peer_addr,
                        Some("Peer sent a file offer request before authentication".to_string()),
                    )
                    .await;
                }
                PeerState::Authenticated { peer_info } => {
                    // Peer is authenticated.
                    // Send a backend event to the frontend with the file offer request
                    // Add the file transfer state to the PeerManager
                    self.backend_event_tx
                        .send(BackendEvent::FileOffer(FileOffer {
                            peer: peer_info.into_connection_info(peer_addr),
                            filename: file_offer.filename.clone(),
                            unique_id: file_offer.unique_id.to_string(),
                            size: file_offer.size,
                        }))
                        .await
                        .expect("Failed to send FileOfferRequest event to the frontend");

                    // Store transfer state
                    self.active_transfers.lock().await.insert(
                        file_offer.unique_id,
                        FileTransferState {
                            unique_id: file_offer.unique_id,
                            peer_addr,
                            direction: FileTransferDirection::Receiving,
                            filename: file_offer.filename,
                            total_size: file_offer.size,
                            bytes_transferred: 0,
                            chunk_len: file_offer.chunk_len,
                            status: FileTransferStatus::InProgress,
                        },
                    );
                }
                PeerState::Disconnecting { .. } => {
                    // Peer is already disconnecting, but they sent a file offer request?
                    // Disconnect the peer
                    self.drop_peer(peer_addr, None).await;
                }
            }
        }
    }
}
