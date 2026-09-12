// 1. We import Rithik's modules here in the main brain
mod commands;

// 2. We create the React test bridge
#[tauri::command]
fn test_rust_bridge() -> String {
    "Zero-Trace Rust Engine is Online and Fully Connected!".to_string()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        // 3. We register ALL commands so React can hear them!
        .invoke_handler(tauri::generate_handler![
            commands::drive_eraser_cmds::run_mock_drive_erasure,
            commands::file_eraser_cmds::test_mock_file_erasure,
            test_rust_bridge
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}