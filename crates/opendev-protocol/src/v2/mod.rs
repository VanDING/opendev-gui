//! V2 protocol types (active development).
//! V2 extends V1 with new methods/fields. V1 clients remain compatible.
//! Started in v0.3.0.

// Placeholder for future v2 types:
// - realtime/voice/*
// - workspace/multi-client/*
// - telegram/*

/// Stub marker — v2 is not yet defined.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct V2Placeholder {
    pub message: String,
}

impl Default for V2Placeholder {
    fn default() -> Self {
        Self { message: "V2 protocol — active development target, not yet defined".into() }
    }
}
