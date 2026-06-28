pub mod version;
pub mod envelope;
pub mod methods;
pub mod events;
pub mod transport;
pub mod transport_tui;
pub mod transport_web;
pub mod transport_workspace;
pub mod v1;
pub mod v2;
pub mod experimental;

// Re-exports for convenience
pub use envelope::{WireEnvelope, RequestFrame, ResponseFrame, NotificationFrame, ErrorFrame, Payload};
pub use methods::Method;
pub use events::Event;
pub use transport::{Transport, EventStream, EventHandle, NegotiatedVersion, ProtocolError};
pub use transport_tui::{TuiInProcessTransport, TuiTransportServer};
pub use transport_web::WebSocketTransport;
#[cfg(unix)]
pub use transport_workspace::unix_socket::UnixSocketTransport;
#[cfg(windows)]
pub use transport_workspace::named_pipe::NamedPipeTransport;
pub use version::{PROTOCOL_VERSION_MAJOR, PROTOCOL_VERSION_MINOR, PROTOCOL_VERSION_PATCH, PROTOCOL_VERSION};
