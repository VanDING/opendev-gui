//! Workspace-mode transport over Unix Domain Sockets (macOS/Linux)
//! or Named Pipes (Windows). Used by the workspace client to connect
//! to a running OpenDev server.

#[cfg(unix)]
pub mod unix_socket {
    use async_trait::async_trait;
    use crate::envelope::{Payload, WireEnvelope, new_request_id};
    use crate::methods::Method;
    use crate::events::Event;
    use crate::transport::{Transport, EventStream, EventHandle, NegotiatedVersion, ProtocolError};
    use crate::version::ProtocolVersion;
    use std::path::PathBuf;
    use tokio::net::UnixStream;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use std::sync::Arc;
    use tokio::sync::Mutex;

    /// Unix Domain Socket transport for workspace mode.
    pub struct UnixSocketTransport {
        socket_path: PathBuf,
        stream: Arc<Mutex<Option<UnixStream>>>,
    }

    impl UnixSocketTransport {
        pub fn new(socket_path: PathBuf) -> Self {
            Self {
                socket_path,
                stream: Arc::new(Mutex::new(None)),
            }
        }

        pub async fn connect(&self) -> Result<(), ProtocolError> {
            let stream = UnixStream::connect(&self.socket_path)
                .await
                .map_err(|e| ProtocolError::Transport(format!("Unix socket connect failed: {}", e)))?;
            let mut guard = self.stream.lock().await;
            *guard = Some(stream);
            Ok(())
        }
    }

    #[async_trait]
    impl Transport for UnixSocketTransport {
        async fn call<P: Payload, R: Payload>(
            &self,
            method: Method,
            params: P,
        ) -> Result<R, ProtocolError> {
            let frame = WireEnvelope::Request(crate::envelope::RequestFrame {
                v: ProtocolVersion::V1_0_0,
                id: new_request_id(),
                src: "workspace-client".into(),
                dst: String::new(),
                method,
                params,
            });

            let json = serde_json::to_string(&frame)
                .map_err(|e| ProtocolError::Internal(e.to_string()))?;

            let mut guard = self.stream.lock().await;
            let stream = guard.as_mut()
                .ok_or(ProtocolError::NotConnected)?;

            stream.write_all(json.as_bytes()).await
                .map_err(|e| ProtocolError::Transport(e.to_string()))?;
            stream.write_all(b"\n").await
                .map_err(|e| ProtocolError::Transport(e.to_string()))?;

            // Read one JSONL line back
            let mut reader = BufReader::new(stream);
            let mut line = String::new();
            reader.read_line(&mut line).await
                .map_err(|e| ProtocolError::Transport(e.to_string()))?;

            let response: WireEnvelope<R> = serde_json::from_str(&line)
                .map_err(|e| ProtocolError::Internal(format!("deserialize: {}", e)))?;

            match response {
                WireEnvelope::Response(frame) => Ok(frame.result),
                WireEnvelope::Error(frame) => Err(ProtocolError::Internal(frame.message)),
                _ => Err(ProtocolError::Internal("unexpected frame type".into())),
            }
        }

        async fn subscribe(&self, _event: Event) -> Result<EventStream, ProtocolError> {
            // Unix socket transport uses JSONL read loop for notifications.
            // For v1, the workspace client reads notifications in a background task.
            Err(ProtocolError::Internal("subscribe not yet implemented for UnixSocketTransport".into()))
        }

        async fn unsubscribe(&self, _handle: EventHandle) -> Result<(), ProtocolError> {
            Ok(()) // no-op for now
        }

        async fn negotiate(&self) -> Result<NegotiatedVersion, ProtocolError> {
            Ok(NegotiatedVersion {
                requested: ProtocolVersion::V1_0_0,
                selected: ProtocolVersion::V1_0_0,
                min_supported: ProtocolVersion::V1_0_0,
                max_supported: ProtocolVersion::V1_0_0,
            })
        }
    }
}

#[cfg(windows)]
pub mod named_pipe {
    use async_trait::async_trait;
    use crate::envelope::{Payload, new_request_id};
    use crate::methods::Method;
    use crate::events::Event;
    use crate::transport::{Transport, EventStream, EventHandle, NegotiatedVersion, ProtocolError};
    use crate::version::ProtocolVersion;
    use std::path::PathBuf;
    use tokio::net::windows::named_pipe::ClientOptions;

    /// Named Pipe transport for workspace mode on Windows.
    pub struct NamedPipeTransport {
        pipe_path: String,
    }

    impl NamedPipeTransport {
        pub fn new(pipe_name: &str) -> Self {
            Self {
                pipe_path: format!(r"\\.\pipe\{}", pipe_name),
            }
        }
    }

    #[async_trait]
    impl Transport for NamedPipeTransport {
        async fn call<P: Payload, R: Payload>(
            &self,
            _method: Method,
            _params: P,
        ) -> Result<R, ProtocolError> {
            // Named pipe implementation — deferred to full Windows support
            Err(ProtocolError::Internal("NamedPipeTransport not yet fully implemented".into()))
        }

        async fn subscribe(&self, _event: Event) -> Result<EventStream, ProtocolError> {
            Err(ProtocolError::Internal("subscribe not yet implemented".into()))
        }

        async fn unsubscribe(&self, _handle: EventHandle) -> Result<(), ProtocolError> {
            Ok(())
        }

        async fn negotiate(&self) -> Result<NegotiatedVersion, ProtocolError> {
            Ok(NegotiatedVersion {
                requested: ProtocolVersion::V1_0_0,
                selected: ProtocolVersion::V1_0_0,
                min_supported: ProtocolVersion::V1_0_0,
                max_supported: ProtocolVersion::V1_0_0,
            })
        }
    }
}
