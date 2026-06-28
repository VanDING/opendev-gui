use opendev_protocol::version::{PROTOCOL_VERSION_MAJOR, PROTOCOL_VERSION_MINOR, ProtocolVersion};

#[test]
fn test_protocol_version_constants() {
    assert_eq!(PROTOCOL_VERSION_MAJOR, 1);
    assert_eq!(PROTOCOL_VERSION_MINOR, 0);
}

#[test]
fn test_version_compatibility() {
    let v1 = ProtocolVersion::V1_0_0;
    let v1_1 = ProtocolVersion { major: 1, minor: 1, patch: 0 };
    let v2 = ProtocolVersion { major: 2, minor: 0, patch: 0 };

    assert!(v1.is_compatible_with(&v1_1));
    assert!(v1_1.is_compatible_with(&v1));
    assert!(!v1.is_compatible_with(&v2));
    assert!(!v2.is_compatible_with(&v1));
}

#[test]
fn test_version_display() {
    assert_eq!(ProtocolVersion::V1_0_0.to_string(), "1.0.0");
}

#[test]
fn test_version_ordering() {
    let v1 = ProtocolVersion::V1_0_0;
    let v1_1 = ProtocolVersion { major: 1, minor: 1, patch: 0 };
    let v2 = ProtocolVersion { major: 2, minor: 0, patch: 0 };

    assert!(v1 < v1_1);
    assert!(v1_1 < v2);
    assert!(v1 < v2);
}
