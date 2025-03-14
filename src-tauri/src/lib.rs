use std::fs::OpenOptions;

use js_api::{
    backend_event::BackendEvent,
    frontend_event::{FrontendEvent, FrontendEventTx},
};
use tokio::sync::mpsc;
use tracing::debug;
use tracing_subscriber::{layer::SubscriberExt, Layer};

pub mod backend;
pub mod js_api;

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
    // File to log to
    let log_file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open("kuaip2p.log")
        .expect("failed to open log file");

    // Tracing (Debug)
    #[cfg(debug_assertions)]
    let subscriber = tracing_subscriber::registry::Registry::default()
        .with(
            tracing_subscriber::fmt::layer()
                .compact()
                .with_ansi(true)
                .with_filter(tracing_subscriber::filter::LevelFilter::TRACE),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(log_file)
                .with_ansi(false)
                .with_filter(tracing_subscriber::filter::LevelFilter::TRACE),
        );

    // Tracing (Release)
    #[cfg(not(debug_assertions))]
    let subscriber = tracing_subscriber::registry::Registry::default()
        .with(
            tracing_subscriber::fmt::layer()
                .compact()
                .with_ansi(true)
                .with_filter(tracing_subscriber::filter::LevelFilter::INFO),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(log_file)
                .with_ansi(false)
                .with_filter(tracing_subscriber::filter::LevelFilter::INFO),
        );

    tracing::subscriber::set_global_default(subscriber).unwrap();

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
        .invoke_handler(tauri::generate_handler![
            greet,
            is_name_cringe,
            js_api::frontend_event::push_frontend_event
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// Main thread private API for pushing backend events to the frontend.
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
