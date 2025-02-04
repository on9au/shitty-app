use std::net::SocketAddr;

use tracing::warn;

use crate::{
    backend::{
        frontend_manager::FrontendManager,
        peer_manager::PeerState,
        protocol::{DisconnectRequest as MessageDisconnectRequest, Message},
    },
    js_api::{
        backend_event::{BackendEvent, BadFrontendEvent},
        frontend_event::{DisconnectRequest, FrontendEvent},
    },
};

impl FrontendManager {
    pub(crate) async fn handle_disconnect_request(
        &self,
        handle_disconnect_request: DisconnectRequest,
    ) {
        // Handle the disconnect request
        // If the frontend is connected, disconnect the frontend
        // If the frontend is not connected, ignore the request

        let peer_addr: SocketAddr = match handle_disconnect_request.ip.parse() {
            Ok(peer_addr) => peer_addr,
            Err(_) => {
                // Invalid IP address
                self.peer_manager
                    .backend_event_tx
                    .send(BackendEvent::BadFrontendEvent(BadFrontendEvent {
                        event: FrontendEvent::DisconnectRequest(handle_disconnect_request),
                        error: "Invalid IP address".to_string(),
                    }))
                    .await
                    .expect("Failed to send BadFrontendEvent event to the backend");
                return;
            }
        };

        let mut peers = self.peer_manager.active_peers.lock().await;

        if let Some(peer) = peers.get_mut(&peer_addr) {
            // Peer is connected
            // Send a `DisconnectRequest` message to the peer
            match peer
                .tx
                .send(Message::DisconnectRequest(MessageDisconnectRequest {
                    message: handle_disconnect_request.message.clone(),
                }))
                .await
            {
                Ok(_) => {
                    // Message sent successfully
                    // Change state to `Disconnecting`
                    peer.state = PeerState::Disconnecting {
                        reason: handle_disconnect_request.message.clone(),
                        peer_info: {
                            match &peer.state {
                                PeerState::Connected { peer_info } => {
                                    if let Some(peer_info) = peer_info {
                                        peer_info.clone()
                                    } else {
                                        // Peer info not set?
                                        self.peer_manager
                                            .drop_peer(
                                                peer_addr,
                                                Some(
                                                    "Peer info not set when handling DisconnectRequest"
                                                        .to_string(),
                                                ),
                                            )
                                            .await;
                                        return;
                                    }
                                }
                                PeerState::Authenticated { peer_info } => peer_info.clone(),
                                PeerState::Disconnecting { .. } => {
                                    // Peer is already disconnecting, but they sent another disconnect request?
                                    // Disconnect the peer
                                    self.peer_manager
                                        .drop_peer(
                                            peer_addr,
                                            Some("Peer is not connected".to_string()),
                                        )
                                        .await;
                                    return;
                                }
                            }
                        },
                    };
                }
                Err(e) => {
                    // Failed to send the message
                    // Disconnect the peer except override the message with the error
                    warn!(
                        ?e,
                        "Failed to send DisconnectRequest message to the peer. Disconnecting the peer with an error message."
                    );
                    self.peer_manager
                        .drop_peer(
                            peer_addr,
                            Some(
                                "Failed to send DisconnectRequest message to the peer".to_string(),
                            ),
                        )
                        .await;
                }
            }
        } else {
            // Peer is not connected
            // Ignore the request
            warn!("Received a DisconnectRequest from a peer that is not connected.");

            // Complain to the frontend
            self.peer_manager
                .backend_event_tx
                .send(BackendEvent::BadFrontendEvent(BadFrontendEvent {
                    event: FrontendEvent::DisconnectRequest(handle_disconnect_request),
                    error: "Peer is not connected".to_string(),
                }))
                .await
                .expect("Failed to send BadFrontendEvent event to the backend");
        }
    }
}
