use std::net::SocketAddr;

use crate::backend::{
    peer_manager::{PeerManager, PeerState},
    protocol::DisconnectRequest,
};

impl PeerManager {
    pub async fn handle_immediate_connection_close(
        &self,
        disconnect_request: DisconnectRequest,
        peer_addr: SocketAddr,
    ) {
        // Peer wants to disconnect immediately (no ack required)

        let mut peers = self.active_peers.lock().await;
        if let Some(peer) = peers.get_mut(&peer_addr) {
            match &peer.state {
                PeerState::Connected { peer_info } => {
                    // Peer wants to disconnect.
                    // Change state to `Disconnecting`
                    // Close the connection
                    peer.state = PeerState::Disconnecting {
                        reason: disconnect_request.message.clone(),
                        peer_info: {
                            if let Some(peer_info) = peer_info {
                                peer_info.clone()
                            } else {
                                // Peer info not set?
                                self.drop_peer(
                                    peer_addr,
                                    "Peer info not set when handling DisconnectRequest"
                                        .to_string()
                                        .into(),
                                )
                                .await;
                                return;
                            }
                        },
                    };

                    // Drop the peer
                    self.drop_peer(peer_addr, None).await;
                }
                PeerState::Disconnecting { .. } => {
                    // Peer is already disconnecting, but they sent another disconnect request?
                    // Disconnect the peer
                    self.drop_peer(peer_addr, None).await;
                }
                PeerState::Authenticated { peer_info } => {
                    // Peer wants to disconnect.
                    // Change state to `Disconnecting`
                    // Close the connection
                    peer.state = PeerState::Disconnecting {
                        reason: disconnect_request.message.clone(),
                        peer_info: peer_info.clone(),
                    };

                    // Drop the peer
                    self.drop_peer(peer_addr, None).await;
                }
            }
        }
    }
}
