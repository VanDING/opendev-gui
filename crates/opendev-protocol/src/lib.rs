pub mod envelope;
pub mod events;
pub mod experimental;
pub mod methods;
pub mod transport;
pub mod transport_tui;
pub mod transport_web;
pub mod transport_workspace;
pub mod v1;
pub mod v2;
pub mod version;

// Re-exports for convenience
pub use envelope::{
    ErrorFrame, NotificationFrame, Payload, RequestFrame, ResponseFrame, WireEnvelope,
};
pub use events::Event;
pub use methods::Method;
pub use transport::{EventHandle, EventStream, NegotiatedVersion, ProtocolError, Transport};
pub use transport_tui::{TuiInProcessTransport, TuiTransportServer};
pub use transport_web::WebSocketTransport;
#[cfg(windows)]
pub use transport_workspace::named_pipe::NamedPipeTransport;
#[cfg(unix)]
pub use transport_workspace::unix_socket::UnixSocketTransport;
pub use version::{
    PROTOCOL_VERSION, PROTOCOL_VERSION_MAJOR, PROTOCOL_VERSION_MINOR, PROTOCOL_VERSION_PATCH,
};
