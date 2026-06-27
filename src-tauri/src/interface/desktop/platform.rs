//! DesktopPlatform trait and Tauri implementation.
//!
//! DesktopPlatform abstracts the Desktop Framework (Tauri, Slint, Wry, etc.)
//! so that Application Services can emit events and manage state without
//! depending on any specific framework.

use serde::Serialize;
use tauri::Emitter;
use tauri::Manager;

/// Desktop Platform abstraction.
///
/// Implementors: TauriPlatform, SlintPlatform, WryPlatform, etc.
/// This is the ONLY place where Desktop Framework types are referenced.
#[async_trait::async_trait]
pub trait DesktopPlatform: Send + Sync + 'static {
    /// Register managed state accessible from Commands.
    fn manage<T: Send + Sync + 'static>(&self, state: T);

    /// Emit an event to the frontend.
    fn emit_event(&self, event: &str, payload: impl Serialize + Clone);

    /// Create a stream channel pair for Data/State Streams.
    fn create_stream<T: Serialize + Send + 'static + Clone>(
        &self,
    ) -> (StreamSender<T>, StreamReceiver<T>);

    /// Get the AppHandle for further operations.
    fn app_handle(&self) -> Option<tauri::AppHandle> {
        None
    }
}

/// Stream sender — sends data to the frontend via the stream.
#[derive(Clone)]
pub struct StreamSender<T> {
    inner: std::sync::Arc<tokio::sync::broadcast::Sender<T>>,
}

impl<T: Clone + Send> StreamSender<T> {
    pub fn send(&self, value: T) -> Result<(), String> {
        self.inner.send(value).map(|_| ()).map_err(|_| "Stream closed".to_string())
    }
}

/// Stream receiver — frontend consumes data from this.
pub struct StreamReceiver<T> {
    inner: tokio::sync::broadcast::Receiver<T>,
}

impl<T: Clone> StreamReceiver<T> {
    pub async fn recv(&mut self) -> Option<T> {
        match self.inner.recv().await {
            Ok(value) => Some(value),
            Err(_) => None,
        }
    }
}

/// Tauri implementation of DesktopPlatform.
pub struct TauriPlatform {
    app: tauri::AppHandle,
}

impl TauriPlatform {
    pub fn new(app: tauri::AppHandle) -> Self {
        Self { app }
    }
}

#[async_trait::async_trait]
impl DesktopPlatform for TauriPlatform {
    fn manage<T: Send + Sync + 'static>(&self, state: T) {
        self.app.manage(state);
    }

    fn emit_event(&self, event: &str, payload: impl Serialize + Clone) {
        let _ = self.app.emit(event, payload);
    }

    fn create_stream<T: Serialize + Send + 'static + Clone>(
        &self,
    ) -> (StreamSender<T>, StreamReceiver<T>) {
        let (tx, _rx) = tokio::sync::broadcast::channel::<T>(256);
        (StreamSender { inner: std::sync::Arc::new(tx) }, StreamReceiver { inner: _rx })
    }

    fn app_handle(&self) -> Option<tauri::AppHandle> {
        Some(self.app.clone())
    }
}
