use js_api::{backend_event::BackendEvent, frontend_event::FrontendEvent};
use js_rs_interop::FrontendEventTx;
use tokio::sync::mpsc;
use tracing::debug;

pub mod backend;
pub mod js_api;
pub mod js_rs_interop;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn is_name_cringe(name: &str) -> bool {
    name.to_lowercase().trim_end().contains("cameron")
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Tracing (Debug)
    #[cfg(debug_assertions)]
    tracing_subscriber::fmt::fmt()
        .with_max_level(tracing::Level::TRACE)
        .init();

    // Tracing (Release)
    #[cfg(not(debug_assertions))]
    tracing_subscriber::fmt::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // Create message passing channels
    // (Frontend Events) Js -> Main Thread -> Tokio
    let (frontend_event_tx, frontend_event_rx) = mpsc::channel::<FrontendEvent>(1);
    // (Backend Events) Tokio -> Main Thread -> Js
    let (backend_event_tx, mut backend_event_rx) = mpsc::channel::<BackendEvent>(1);

    tauri::Builder::default()
        // Store the async process input transmitter in the Tauri state
        .manage(FrontendEventTx::new(frontend_event_tx))
        // Set up the Tokio runtime
        .setup(|app| {
            // Run the main async backend process
            tauri::async_runtime::spawn(async move {
                backend::init(frontend_event_rx, backend_event_tx).await
            });

            // Message passing from Tokio -> main thread to js
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                loop {
                    // Send messages from Tokio to js
                    if let Some(event) = backend_event_rx.recv().await {
                        send_backend_event(&app_handle, event);
                    }
                }
            });

            Ok(())
        })
        // Do the rest of the Tauri setup
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, is_name_cringe])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// Main thread API for pushing backend events to the frontend.
///
/// - The event emitted is `backend_event`.
fn send_backend_event<R: tauri::Runtime>(
    app_handle: &(impl tauri::Manager<R> + tauri::Emitter<R>),
    event: BackendEvent,
) {
    debug!(?event, "Backend Event Received");
    app_handle
        .emit("backend_event", event)
        .expect("failed to emit event");
}
