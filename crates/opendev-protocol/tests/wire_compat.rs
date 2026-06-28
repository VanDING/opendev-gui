use opendev_protocol::envelope::{ErrorFrame, NotificationFrame, RequestFrame, WireEnvelope};
use opendev_protocol::events::Event;
use opendev_protocol::methods::Method;
use opendev_protocol::version::ProtocolVersion;
use serde_json::Value;

/// V1 wire format: Request round-trip.
#[test]
fn test_request_round_trip() {
    let frame = RequestFrame {
        v: ProtocolVersion::V1_0_0,
        id: "0193f6b4-1234-7abc-8901-234567890abc".into(),
        src: "0193f6b4-client-7abc-8901-234567890abc".into(),
        dst: String::new(),
        method: Method::SessionStart,
        params: serde_json::json!({"title": "test session"}),
    };

    let envelope: WireEnvelope<Value> = WireEnvelope::Request(frame);
    let json = serde_json::to_string(&envelope).expect("serialize should succeed");

    // Verify JSON structure
    let parsed: serde_json::Value = serde_json::from_str(&json).expect("should be valid JSON");
    assert_eq!(parsed["kind"], "request");
    assert!(parsed["id"].is_string());
    assert_eq!(parsed["method"], "session/start");
    assert_eq!(parsed["v"]["major"], 1);
    assert_eq!(parsed["v"]["minor"], 0);
    assert_eq!(parsed["params"]["title"], "test session");

    // Deserialize back
    let _decoded: WireEnvelope<Value> =
        serde_json::from_str(&json).expect("deserialize should succeed");
}

/// V1 wire format: Notification round-trip.
#[test]
fn test_notification_round_trip() {
    let frame = NotificationFrame {
        v: ProtocolVersion::V1_0_0,
        seq: 42,
        src: "server-id".into(),
        event: Event::MessageChunked,
        data: serde_json::json!({"session_id": "s1", "content": "Hello"}),
    };

    let envelope: WireEnvelope<Value> = WireEnvelope::Notification(frame);
    let json = serde_json::to_string(&envelope).expect("serialize should succeed");

    let parsed: serde_json::Value = serde_json::from_str(&json).expect("should be valid JSON");
    assert_eq!(parsed["kind"], "notification");
    assert_eq!(parsed["seq"], 42);
    assert_eq!(parsed["event"], "message/chunked");

    let _decoded: WireEnvelope<Value> =
        serde_json::from_str(&json).expect("deserialize should succeed");
}

/// V1 wire format: Error frame.
#[test]
fn test_error_frame_round_trip() {
    let frame = ErrorFrame {
        v: ProtocolVersion::V1_0_0,
        id: Some("req-123".into()),
        src: "server".into(),
        dst: "client".into(),
        code: -32601,
        message: "Method not found".into(),
        data: None,
    };

    let envelope: WireEnvelope<Value> = WireEnvelope::Error(frame);
    let json = serde_json::to_string(&envelope).expect("serialize should succeed");

    let parsed: serde_json::Value = serde_json::from_str(&json).expect("should be valid JSON");
    assert_eq!(parsed["kind"], "error");
    assert_eq!(parsed["code"], -32601);
    assert_eq!(parsed["message"], "Method not found");
}

/// V1 wire format: version field is always present.
#[test]
fn test_version_field_present() {
    let frame = RequestFrame {
        v: ProtocolVersion::V1_0_0,
        id: "test-1".into(),
        src: "client-1".into(),
        dst: String::new(),
        method: Method::SessionList,
        params: serde_json::json!({}),
    };
    let envelope: WireEnvelope<Value> = WireEnvelope::Request(frame);
    let json = serde_json::to_string(&envelope).unwrap();
    assert!(json.contains("\"major\":1"), "Version major must be in output: {}", json);
    assert!(json.contains("\"minor\":0"), "Version minor must be in output: {}", json);
}
