use std::net::SocketAddr;

use crate::{
    backend::{
        frontend_manager::FrontendManager,
        peer_manager::PeerState,
        protocol::{ConnectionPermit, ConnectionResponse, EcdsaConnectionInfo, Message},
    },
    js_api::frontend_event::ConnectionRequestResponse,
};

impl FrontendManager {
    pub(crate) async fn handle_connection_request_response(
        &mut self,
        connection_request_response: ConnectionRequestResponse,
    ) {
        // Handle the connection request response
        // If accepted, change state to `Authenticated` and send a `ConnectResponse` with `Permit` message
        // If rejected, send a `ConnectResponse` with `Deny` message

        let peer_addr: SocketAddr = connection_request_response.ip.parse().unwrap();

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
                            identitiy: EcdsaConnectionInfo {
                                public_key: vec![], // TODO: Implement
                                signature: vec![],  // TODO: Implement
                                nonce: vec![],      // TODO: Implement
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
        }
    }
}
