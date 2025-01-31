//! # Tauri JS and Backend API
//!
//! This module contains the Tauri JS API for the application.
//!
//! The API is split into two parts:
//! - `backend_event`: Events that occur in the backend and should be sent to the frontend.
//! - `frontend_event`: Events that occur in the frontend and should be sent to the backend.
//!
//! ## JS Calling into Rust steps:
//!
//! - JS calls a Tauri command [frontend_event::push_frontend_event].
//!   - The main thread receives the command and sends the [frontend_event::FrontendEvent] to the backend via the [crate::FrontendEventTx] mpsc channel.
//!   - The [frontend_event::FrontendEvent] is received by the tokio-based backend and processed.
//!
//! **The function name is `push_frontend_event`.**
//!
//! ## Rust Calling into JS steps:
//!
//! - The tokio backend sends a [backend_event::BackendEvent] to backend_event_tx mpsc channel.
//!   - The main thread receives the [backend_event::BackendEvent] and sends it to the frontend.
//!   - The frontend receives the event and processes it.
//!
//! **The event is emitted as `backend_event` (string).**
//!
//! ## JS API JavaScript-ish Pseudo Code Example Thing[]:
//!
//! ```javascript
//! // Backend Event Listener
//! // Uses the Tauri `listen` function to listen for the Rust event `backend_event`.
//! await listen('backend_event', (event) => {
//!   console.log("backend event: " + event)
//!   let input = event.payload
//!   inputs.value.push({ timestamp: Date.now(), message: input })
//! })
//!
//! // Frontend Event Emitter
//! // Uses the Tauri `invoke` function to call the API function `push_frontend_event`.
//! await invoke('push_frontend_event', { /* Data */ })
//! ```

pub mod backend_event;
pub mod frontend_event;
