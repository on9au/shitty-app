use std::net::SocketAddr;

use crate::backend::peer_manager::PeerManager;

impl PeerManager {
    /// # Message Handler: `DisconnectAck`
    ///
    /// Handle a disconnect ack.
    pub async fn handle_disconnect_ack(&self, peer_addr: SocketAddr) {
        // Peer has acknowledged the disconnect request.
        // Remove the peer from the active peers list.
        self.drop_peer(peer_addr, None).await;
    }
}
