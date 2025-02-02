use std::net::SocketAddr;

use tracing::warn;

use crate::backend::{
    peer_manager::{PeerManager, PeerState},
    protocol::{DisconnectRequest, Message},
};

impl PeerManager {
    /// # Message Handler: `DisconnectRequest`
    ///
    /// Handle a disconnect request.
    pub async fn handle_disconnect_request(
        &self,
        disconnect_request: DisconnectRequest,
        peer_addr: SocketAddr,
    ) {
        // Peer wants to disconnect from us
        // Change state to `Disconnected`
        // Send a `DisconnectAck` message
        // Close the connection

        let mut peers = self.active_peers.lock().await;
        if let Some(peer) = peers.get_mut(&peer_addr) {
            match &peer.state {
                PeerState::Connected { peer_info } => {
                    // Peer wants to disconnect.
                    // Change state to `Disconnecting`
                    // Send a `DisconnectAck` message
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

                    // Send a `DisconnectAck` message
                    match peer.tx.send(Message::DisconnectAck).await {
                        Ok(_) => {
                            // Message sent successfully
                            // Close the connection
                            self.drop_peer(peer_addr, None).await;
                        }
                        Err(e) => {
                            // Failed to send the message
                            // Disconnect the peer except override the message with the error
                            warn!(
                                    "Failed to send `DisconnectAck` message to peer {}. Disconnecting peer. Reason: {}. Error: {}",
                                    peer_addr, disconnect_request.message.as_deref().unwrap_or("No reason provided"), e
                                );
                            self.drop_peer(peer_addr, e.to_string().into()).await;
                        }
                    };
                }
                PeerState::Disconnecting { .. } => {
                    // Peer is already disconnecting, but they sent another disconnect request?
                    // Disconnect the peer
                    self.drop_peer(peer_addr, None).await;
                }
                PeerState::Authenticated { peer_info } => {
                    // Peer wants to disconnect.
                    // Change state to `Disconnecting`
                    // Send a `DisconnectAck` message
                    // Close the connection
                    peer.state = PeerState::Disconnecting {
                        reason: disconnect_request.message.clone(),
                        peer_info: peer_info.clone(),
                    };

                    // Send a `DisconnectAck` message
                    match peer.tx.send(Message::DisconnectAck).await {
                        Ok(_) => {
                            // Message sent successfully
                            // Close the connection
                            self.drop_peer(peer_addr, None).await;
                        }
                        Err(e) => {
                            // Failed to send the message
                            // Disconnect the peer except override the message with the error
                            warn!(
                                    "Failed to send `DisconnectAck` message to peer {}. Disconnecting peer. Reason: {}. Error: {}",
                                    peer_addr, disconnect_request.message.as_deref().unwrap_or("No reason provided"), e
                                );
                            self.drop_peer(peer_addr, e.to_string().into()).await;
                        }
                    };
                }
            }
        }
    }
}
