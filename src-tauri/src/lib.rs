use js_rs_interop::{rs2js, AsyncProcInputTx};
use tokio::sync::mpsc;

pub mod js_rs_interop;
pub mod protocol;

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
    // Create message passing channels
    let (async_proc_input_tx, async_proc_input_rx) = tokio::sync::mpsc::channel(1);
    let (async_proc_output_tx, mut async_proc_output_rx) = tokio::sync::mpsc::channel(1);

    tauri::Builder::default()
        .manage(AsyncProcInputTx::new(async_proc_input_tx))
        .setup(|app| {
            tauri::async_runtime::spawn(async move {
                async_process_model(async_proc_input_rx, async_proc_output_tx).await
            });

            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                loop {
                    if let Some(output) = async_proc_output_rx.recv().await {
                        rs2js(&app_handle, output);
                    }
                }
            });

            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, is_name_cringe])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

async fn async_process_model(
    mut input_rx: mpsc::Receiver<String>,
    output_tx: mpsc::Sender<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    while let Some(input) = input_rx.recv().await {
        let output = input;
        output_tx.send(output).await?;
    }

    Ok(())
}
