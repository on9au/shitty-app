use std::net::SocketAddr;

use crate::{
    backend::{
        frontend_manager::FrontendManager,
        peer_manager::PeerState,
        protocol::{
            ConnectionInfo, ConnectionPermit, ConnectionResponse, EcdsaConnectionInfo, Message,
        },
    },
    js_api::{
        backend_event::{BackendEvent, BadFrontendEvent},
        frontend_event::{ConnectionRequestResponse, FrontendEvent},
    },
};

impl FrontendManager {
    pub(crate) async fn handle_connection_request_response(
        &mut self,
        connection_request_response: ConnectionRequestResponse,
    ) {
        // Handle the connection request response
        // If accepted, change state to `Authenticated` and send a `ConnectResponse` with `Permit` message
        // If rejected, send a `ConnectResponse` with `Deny` message

        let peer_addr: SocketAddr = match connection_request_response.ip.parse() {
            Ok(peer_addr) => peer_addr,
            Err(_) => {
                // Invalid IP address
                self.peer_manager
                    .backend_event_tx
                    .send(BackendEvent::BadFrontendEvent(BadFrontendEvent {
                        event: FrontendEvent::ConnectionRequestResponse(
                            connection_request_response,
                        ),
                        error: "Invalid IP address".to_string(),
                    }))
                    .await
                    .expect("Failed to send BadFrontendEvent event to the backend");
                return;
            }
        };

        let mut peers = self.peer_manager.active_peers.lock().await;

        if let Some(peer) = peers.get_mut(&peer_addr) {
            if connection_request_response.accept {
                // Connection accepted, change state to `Authenticated` and send a `ConnectResponse` with `Permit` message
                // peer.state = PeerState::Authenticated;
                if let PeerState::Connected { peer_info } = &peer.state {
                    let peer_info = peer_info.as_ref().expect(
                        "Peer info was not set when handling the connection request response???",
                    );
                    let connection_response = ConnectionResponse {
                        permit: ConnectionPermit::Permit {
                            identitiy: ConnectionInfo {
                                name: "todo!".to_string(),
                                backend_version: env!("CARGO_PKG_VERSION").to_string(),
                                identitiy: EcdsaConnectionInfo {
                                    public_key: vec![], // TODO: Implement this
                                    signature: vec![],  // TODO: Implement this
                                    nonce: vec![],      // TODO: Implement this
                                },
                            },
                        },
                        message: connection_request_response.message.clone(),
                    };

                    // Update state to `Authenticated`
                    peer.state = PeerState::Authenticated {
                        peer_info: peer_info.clone(),
                    };

                    // Send the connection response
                    peer.tx
                        .send(Message::ConnectResponse(connection_response))
                        .await
                        .expect("Failed to send ConnectResponse message to the peer");
                }
            } else {
                // Connection rejected, send a `ConnectResponse` with `Deny` message
                // This packet is treated as a disconnect request
                match &peer.state {
                    PeerState::Connected { peer_info } => {
                        let peer_info = peer_info.as_ref().expect("Peer info was not set when handling the connection request response???");
                        let connection_response = ConnectionResponse {
                            permit: ConnectionPermit::Deny,
                            message: connection_request_response.message.clone(),
                        };

                        let reason = {
                            if connection_request_response.message.is_none() {
                                "Connection rejected by the user".to_string().into()
                            } else {
                                connection_request_response.message.clone()
                            }
                        };

                        // Update state to `Disconnecting`
                        peer.state = PeerState::Disconnecting {
                            reason: reason.clone(),
                            peer_info: peer_info.clone(),
                        };

                        // Send the connection response
                        peer.tx
                            .send(Message::ConnectResponse(connection_response))
                            .await
                            .expect("Failed to send ConnectResponse message to the peer");
                    }
                    _ => {
                        // Peer is in an invalid state.
                        // Drop the peer.
                        self.peer_manager
                            .drop_peer(
                                peer_addr,
                                "Peer is not in the connecting state".to_string().into(),
                            )
                            .await;
                    }
                }
            }
        } else {
            // Peer that frontend is trying to respond to does not exist
            self.peer_manager
                .backend_event_tx
                .send(BackendEvent::BadFrontendEvent(BadFrontendEvent {
                    event: FrontendEvent::ConnectionRequestResponse(connection_request_response),
                    error: "Peer does not exist".to_string(),
                }))
                .await
                .expect("Failed to send BadFrontendEvent event to the backend");
        }
    }
}
