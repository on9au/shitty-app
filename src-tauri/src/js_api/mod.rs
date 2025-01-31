//! # Tauri JS API
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
//! ## Rust Calling into JS steps:
//!
//! - The tokio backend sends a [backend_event::BackendEvent] to backend_event_tx mpsc channel.
//!   - The main thread receives the [backend_event::BackendEvent] and sends it to the frontend.
//!   - The frontend receives the event and processes it.
//!

pub mod backend_event;
pub mod frontend_event;
