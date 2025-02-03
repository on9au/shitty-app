use std::net::SocketAddr;

use base64::{prelude::BASE64_STANDARD, Engine};

use crate::{
    backend::{
        peer_manager::{PeerInfo, PeerManager, PeerState},
        protocol::{ConnectionInfo, Message},
    },
    js_api::backend_event::{self, BackendEvent},
};

impl PeerManager {
    /// # Message Handler: `ConnectRequest`
    ///
    /// Handle a connect request.
    pub async fn handle_connect_request(
        &self,
        connection_info: ConnectionInfo,
        peer_addr: SocketAddr,
    ) {
        // Peer wants to connect to us
        // Prompt the frontend to accept or reject the connection
        // If accepted, change state to `Authenticated` and send a `ConnectResponse` message

        // Prompt the frontend to accept or reject the connection
        self.backend_event_tx
            .send(BackendEvent::ConnectRequest(
                backend_event::ConnectionInfo {
                    name: connection_info.name.clone(),
                    ip: peer_addr.to_string(),
                    backend_version: connection_info.backend_version.clone(),
                    identitiy: BASE64_STANDARD.encode(connection_info.identitiy.public_key.clone()),
                },
            ))
            .await
            .expect("Failed to send ConnectRequest event to the frontend");

        // Update the state of the peer to include the connection info
        {
            let mut peers = self.active_peers.lock().await;
            if let Some(peer) = peers.get_mut(&peer_addr) {
                peer.state = PeerState::Connected {
                    peer_info: Some(PeerInfo {
                        name: connection_info.name,
                        backend_version: connection_info.backend_version,
                        ecdsa_public_key: connection_info.identitiy.public_key,
                    }),
                };
            }
        }

        // This is the most we can do for now. The frontend will respond with a `ConnectResponse` message, and
        // the specific handler will continue the process.
        // Let's just begin the keep-alive ping-pong to keep the connection alive.
        {
            let mut peers = self.active_peers.lock().await;
            if let Some(peer) = peers.get_mut(&peer_addr) {
                peer.tx
                    .send(Message::KeepAlive)
                    .await
                    .expect("Failed to send KeepAlive message to the peer");
            }
        }
    }
}
