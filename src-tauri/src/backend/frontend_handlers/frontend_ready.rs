use tracing::warn;

use crate::{
    backend::frontend_manager::FrontendManager,
    js_api::{
        backend_event::{BackendEvent, BadFrontendEvent},
        frontend_event::{BackendStartupConfig, FrontendEvent},
    },
};

impl FrontendManager {
    pub(crate) async fn handle_frontend_ready(
        &mut self,
        backend_startup_config: BackendStartupConfig,
    ) {
        // Unexpected event. We only expect this event once at the start of the backend.
        // Log a warning, inform frontend, and ignore the event.

        warn!("Received unexpected `FrontendReady` event after the backend has started. Ignoring the event.");

        // Send an event to the frontend to inform the user that the backend is already running.
        self.peer_manager
            .backend_event_tx
            .send(BackendEvent::BadFrontendEvent(BadFrontendEvent {
                event: FrontendEvent::FrontendReady(backend_startup_config),
                error: "Backend is already running".to_string(),
            }))
            .await
            .expect("Failed to send BadFrontendEvent event to the backend");

        // Ignore the event
    }
}
