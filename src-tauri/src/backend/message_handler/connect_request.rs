use std::net::SocketAddr;

use base64::{prelude::BASE64_STANDARD, Engine};

use crate::{
    backend::{
        peer_manager::PeerManager,
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
                    name: connection_info.name,
                    ip: peer_addr.to_string(),
                    backend_version: connection_info.backend_version,
                    identitiy: BASE64_STANDARD.encode(connection_info.identitiy.public_key),
                },
            ))
            .await
            .expect("Failed to send ConnectRequest event to the frontend");

        // This is the most we can do for now. The frontend will respond with a `ConnectResponse` message, and
        // the specific handler will continue the process.
        // Let's just begin the keep-alive ping-pong to keep the connection alive.
        self.active_peers
            .lock()
            .await
            .get_mut(&peer_addr)
            .expect("Peer not found in active peers list. Unreachable state.")
            .tx
            .send(Message::KeepAlive)
            .await
            .expect("Failed to send KeepAlive message to the peer");
    }
}
