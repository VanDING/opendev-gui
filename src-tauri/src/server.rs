use std::net::TcpListener as StdTcpListener;
use std::path::Path;

use opendev_config::ConfigLoader;
use opendev_config::Paths;
use opendev_history::SessionManager;
use opendev_http::UserStore;
use opendev_web::server::build_app;
use opendev_web::state::AppState;

pub struct ServerHandle {
    pub port: u16,
    pub shutdown_tx: tokio::sync::oneshot::Sender<()>,
}

pub fn build_server(working_dir: &Path) -> Result<ServerHandle, String> {
    let paths = Paths::new(Some(working_dir.to_path_buf()));

    // ── Config & Session ───────────────────────────────────────────
    let config = ConfigLoader::load(&paths.global_settings(), &paths.project_settings())
        .map_err(|e| format!("Failed to load config: {}", e))?;

    let session_dir = paths.global_sessions_dir();
    std::fs::create_dir_all(&session_dir)
        .map_err(|e| format!("Failed to create session dir: {}", e))?;
    let session_manager = SessionManager::new(session_dir)
        .map_err(|e| format!("Failed to init session manager: {}", e))?;

    let user_store_dir = paths.data_dir().join("users");
    std::fs::create_dir_all(&user_store_dir)
        .map_err(|e| format!("Failed to create user store dir: {}", e))?;
    let user_store =
        UserStore::new(user_store_dir).map_err(|e| format!("Failed to init user store: {}", e))?;

    let model_registry = opendev_config::ModelRegistry::new();

    // ── App State ──────────────────────────────────────────────────
    let state = AppState::new(
        session_manager,
        config,
        working_dir.to_string_lossy().to_string(),
        user_store,
        model_registry,
    );

    let router = build_app(state, None);

    let std_listener =
        StdTcpListener::bind("127.0.0.1:0").map_err(|e| format!("Failed to bind: {}", e))?;
    let port = std_listener
        .local_addr()
        .map(|a| a.port())
        .map_err(|e| format!("Failed to get port: {}", e))?;
    std_listener.set_nonblocking(true).map_err(|e| format!("Failed to set nonblocking: {}", e))?;
    let listener = tokio::net::TcpListener::from_std(std_listener)
        .map_err(|e| format!("Failed to create tokio listener: {}", e))?;

    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();

    tokio::spawn(async move {
        axum::serve(listener, router)
            .with_graceful_shutdown(async {
                let _ = shutdown_rx.await;
            })
            .await
            .ok();
    });

    Ok(ServerHandle { port, shutdown_tx })
}
