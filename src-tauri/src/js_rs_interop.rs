use std::ops::{Deref, DerefMut};

use crate::js_api::frontend_event::FrontendEvent;

/// Async Process Input Transmitter
///
/// Main Thread -> Tokio
pub struct AsyncProcInputTx {
    inner: tokio::sync::Mutex<tokio::sync::mpsc::Sender<FrontendEvent>>,
}

impl AsyncProcInputTx {
    pub fn new(tx: tokio::sync::mpsc::Sender<FrontendEvent>) -> Self {
        Self {
            inner: tokio::sync::Mutex::new(tx),
        }
    }
}

impl Deref for AsyncProcInputTx {
    type Target = tokio::sync::Mutex<tokio::sync::mpsc::Sender<FrontendEvent>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for AsyncProcInputTx {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

// /// This function is called from JavaScript to send a message to Rust-Tokio.
// #[tauri::command]
// pub async fn js2rs(
//     message: String,
//     state: tauri::State<'_, AsyncProcInputTx>,
// ) -> Result<(), String> {
//     info!(?message, "js2rs");
//     let async_process_input_tx = state.lock().await;
//     async_process_input_tx
//         .send(message)
//         .await
//         .map_err(|e| e.to_string())
// }

// /// This function is called from Rust-Tokio to send a message to JavaScript.
// pub fn rs2js<R: tauri::Runtime>(
//     app_handle: &(impl tauri::Manager<R> + tauri::Emitter<R>),
//     message: String,
// ) {
//     info!(?message, "rs2js");
//     app_handle
//         .emit("rs2js", format!("{{\"message\": \"{}\"}}", message))
//         .expect("failed to emit event");
// }
