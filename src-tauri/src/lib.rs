use js_api::{backend_event::BackendEvent, frontend_event::FrontendEvent};
use js_rs_interop::AsyncProcInputTx;
use tokio::sync::mpsc;
use tracing::info;

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
        .manage(AsyncProcInputTx::new(frontend_event_tx))
        // Set up the Tokio runtime
        .setup(|_app| {
            // Run the main async backend process
            tauri::async_runtime::spawn(async move {
                backend::init(frontend_event_rx, backend_event_tx).await
            });

            // Message passing from Tokio to js
            // let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                loop {
                    // Send messages from Tokio to js
                    if let Some(output) = backend_event_rx.recv().await {
                        info!(?output, "Sending message to js");
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

// async fn async_process_model(
//     mut input_rx: mpsc::Receiver<String>,
//     output_tx: mpsc::Sender<String>,
// ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
//     // While there is input from main thread, send back to main thread
//     while let Some(input) = input_rx.recv().await {
//         let output = input;
//         output_tx.send(output).await?;
//     }

//     Ok(())
// }
