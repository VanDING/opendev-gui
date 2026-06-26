#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod server;

use std::sync::Mutex;
use tauri::Manager;

struct AppState {
    server: Mutex<Option<server::ServerHandle>>,
}

fn main() {
    tauri::Builder::default()
        .manage(AppState { server: Mutex::new(None) })
        .setup(|app| {
            let working_dir =
                std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));

            match server::build_server(&working_dir) {
                Ok(handle) => {
                    let port = handle.port;

                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.eval(&format!("window.__OPENDEV_PORT__ = {}", port));
                    }

                    let state = app.state::<AppState>();
                    *state.server.lock().unwrap_or_else(|e| e.into_inner()) = Some(handle);

                    println!("OpenDev server started on http://127.0.0.1:{}", port);
                }
                Err(e) => {
                    eprintln!("Failed to start OpenDev server: {}", e);
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.eval(&format!(
                            "document.body.innerHTML = '<h1>Server Error</h1><pre>{}</pre>'",
                            e.replace('\'', "\\'")
                        ));
                    }
                }
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::Destroyed = event {
                let app = window.app_handle();
                let state = app.state::<AppState>();
                if let Some(handle) = state.server.lock().unwrap_or_else(|e| e.into_inner()).take()
                {
                    let _ = handle.shutdown_tx.send(());
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running OpenDev Desktop");
}
