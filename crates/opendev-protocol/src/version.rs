/// Protocol version constants. V1 is frozen at v0.2.0 GA.
pub const PROTOCOL_VERSION_MAJOR: u16 = 1;
pub const PROTOCOL_VERSION_MINOR: u16 = 0;
pub const PROTOCOL_VERSION_PATCH: u16 = 0;
pub const PROTOCOL_VERSION: &str = "1.0.0";

/// Represents a protocol version for wire serialization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct ProtocolVersion {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
}

impl ProtocolVersion {
    pub const V1_0_0: Self = Self { major: 1, minor: 0, patch: 0 };

    pub fn is_compatible_with(&self, other: &Self) -> bool {
        self.major == other.major
    }
}

impl std::fmt::Display for ProtocolVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl Default for ProtocolVersion {
    fn default() -> Self {
        Self::V1_0_0
    }
}
