use std::net::SocketAddr;

use crate::backend::{peer_manager::PeerManager, protocol::Message};

impl PeerManager {
    /// # Message Handler: `KeepAlive`
    ///
    /// Handle a keep-alive message.
    pub async fn handle_keep_alive(&self, peer_addr: SocketAddr) {
        // Send a keep-alive message back to the peer
        // after a short delay (10 seconds) to prevent TCP connections from timing out
        // (Time out is 30 seconds)
        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
        let peers = self.active_peers.lock().await;
        // If the peer is not found, they have already disconnected, return.
        if let Some(peer) = peers.get(&peer_addr) {
            peer.tx
                .send(Message::KeepAlive)
                .await
                .expect("Failed to send KeepAlive message to the peer");
        }
    }
}
