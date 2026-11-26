mod commands;
mod domain;
mod error;
mod media;
mod state;

use tauri::{Manager, RunEvent};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .manage(state::AppState::default())
        .invoke_handler(tauri::generate_handler![
            commands::video::probe_video,
            commands::video::detect_encoders,
            commands::video::suggest_output,
            commands::video::estimate_size,
            commands::compress::compress,
            commands::compress::cancel_compression,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app, event| {
            if let RunEvent::Exit = event {
                let state = app.state::<state::AppState>();
                state.next_estimate();
                if let Some(job) = state.current_job() {
                    job.shutdown();
                }
            }
        });
}
