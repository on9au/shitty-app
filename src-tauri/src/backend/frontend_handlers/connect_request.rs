use std::net::SocketAddr;

use tracing::{debug, warn};

use crate::{
    backend::{
        frontend_manager::FrontendManager,
        protocol::{ConnectionInfo, EcdsaConnectionInfo, Message},
    },
    js_api::{
        backend_event::{BackendEvent, BadFrontendEvent},
        frontend_event::{ConnectRequest, FrontendEvent},
    },
};

impl FrontendManager {
    pub(crate) async fn handle_connect_request(&mut self, connect_request: ConnectRequest) {
        // Open a connection to the peer
        // If successful, send a `ConnectionRequest` to the peer and wait until the peer responds.

        // Parse the IP address
        let peer_addr: SocketAddr = match connect_request.ip.parse() {
            Ok(peer_addr) => peer_addr,
            Err(_) => {
                // Invalid IP address
                self.peer_manager
                    .backend_event_tx
                    .send(BackendEvent::BadFrontendEvent(BadFrontendEvent {
                        event: FrontendEvent::ConnectRequest(connect_request),
                        error: "Invalid IP address".to_string(),
                    }))
                    .await
                    .expect("Failed to send BadFrontendEvent event to the backend");
                return;
            }
        };

        match self.peer_manager.connect(peer_addr).await {
            Ok(_) => {
                // Connection successful
                // Send a `ConnectionRequest` to the peer
                // Retry 20 times if the peer is not found in the active peers list (500ms * 20 = 10s timeout)
                // Note that we drop the lock after each iteration to prevent deadlocks.
                let mut success = false;
                for _ in 0..20 {
                    let peers = self.peer_manager.active_peers.lock().await;
                    if let Some(peer) = peers.get(&peer_addr) {
                        peer.tx
                            .send(Message::ConnectRequest(ConnectionInfo {
                                name: "todo!".to_string(),
                                backend_version: env!("CARGO_PKG_VERSION").to_string(),
                                identitiy: EcdsaConnectionInfo {
                                    public_key: vec![], // todo!(),
                                    signature: vec![],  // todo!(),
                                    nonce: vec![],      // todo!(),
                                },
                            }))
                            .await
                            .expect("Failed to send ConnectRequest message to the peer");
                        success = true;
                        break;
                    }
                    // Wait for a bit before trying again (500ms)
                    debug!(
                        "Failed to find the peer {} in the active peers list. Retrying... after 500ms",
                        peer_addr
                    );
                    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                }

                if !success {
                    // Peer not found in the active peers list
                    // Log a warning, inform frontend, and ignore the event.
                    warn!("Failed to find the peer in the active peers list. Ignoring the event.");

                    // Send an event to the frontend to inform the user that the connection failed.
                    self.peer_manager
                        .backend_event_tx
                        .send(BackendEvent::BadFrontendEvent(BadFrontendEvent {
                            event: FrontendEvent::ConnectRequest(connect_request),
                            error: "Peer not found in the active peers list".to_string(),
                        }))
                        .await
                        .expect("Failed to send BadFrontendEvent event to the backend");
                }
            }
            Err(e) => {
                // Connection failed
                // Log a warning, inform frontend, and ignore the event.
                warn!(?e, "Failed to connect to the peer. Ignoring the event.");

                // Send an event to the frontend to inform the user that the connection failed.
                self.peer_manager
                    .backend_event_tx
                    .send(BackendEvent::BadFrontendEvent(BadFrontendEvent {
                        event: FrontendEvent::ConnectRequest(connect_request),
                        error: "Failed to connect to the peer".to_string(),
                    }))
                    .await
                    .expect("Failed to send BadFrontendEvent event to the backend");
            }
        };
    }
}
