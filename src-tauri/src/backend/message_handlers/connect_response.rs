use std::net::SocketAddr;

use crate::{
    backend::{
        peer_manager::{PeerInfo, PeerManager, PeerState},
        protocol::{ConnectionPermit, ConnectionResponse, Message},
    },
    js_api::backend_event::{BackendEvent, ConnectionRequestResponse},
};

impl PeerManager {
    pub async fn handle_connect_response(
        &self,
        connect_response: ConnectionResponse,
        peer_addr: SocketAddr,
    ) {
        // Peer has responded to the connection request.
        // If accepted, change state to `Authenticated` and send a `ConnectResponse` message
        // If rejected, reply with a `DisconnectAck` message and close the connection

        let mut peers = self.active_peers.lock().await;
        if let Some(peer) = peers.get_mut(&peer_addr) {
            match connect_response.permit {
                ConnectionPermit::Permit { identitiy } => {
                    // Connection accepted, change state to `Authenticated` and notify frontend

                    if let PeerState::Connected { .. } = &mut peer.state {
                        // Update the peer state to `Authenticated`
                        peer.state = PeerState::Authenticated {
                            peer_info: PeerInfo {
                                name: identitiy.name,
                                ecdsa_public_key: identitiy.identitiy.public_key,
                                backend_version: identitiy.backend_version,
                            },
                        };

                        // Send an event to the frontend to notify the user that the connection was accepted.
                        self.backend_event_tx
                            .send(BackendEvent::ConnectionRequestResponse(
                                ConnectionRequestResponse {
                                    accept: true,
                                    ip: peer_addr.to_string(),
                                    reason: None,
                                },
                            ))
                            .await
                            .expect(
                                "Failed to send ConnectionRequestAccepted event to the frontend",
                            );
                    } else {
                        // Unexpected state. Disconnect the peer
                        self.drop_peer(
                            peer_addr,
                            "Unexpected state. Disconnecting peer.".to_string().into(),
                        )
                        .await;
                    }
                }
                ConnectionPermit::Deny => {
                    // Connection rejected, treat as a disconnect request.

                    // Send an event to the frontend to notify the user that the connection was rejected.
                    self.backend_event_tx
                        .send(BackendEvent::ConnectionRequestResponse(
                            ConnectionRequestResponse {
                                accept: false,
                                ip: peer_addr.to_string(),
                                reason: connect_response.message.clone(),
                            },
                        ))
                        .await
                        .expect("Failed to send ConnectionRequestRejected event to the frontend");

                    // Reply with a `DisconnectAck` message and close the connection.
                    peer.tx
                        .send(Message::DisconnectAck)
                        .await
                        .expect("Failed to send DisconnectAck message to peer");

                    // Close the connection
                    self.drop_peer(peer_addr, None).await;
                }
            }
        }
    }
}
