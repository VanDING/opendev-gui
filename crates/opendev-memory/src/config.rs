use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct MemoryConfig {
    pub db_path: PathBuf,
    pub write_gate_min_count: usize,
    pub write_gate_max_age: Duration,
    pub short_term_max_kv: usize,
    pub short_term_max_turns: usize,
    pub short_term_ttl: Duration,
    pub cascade_min_count: usize,
    pub cascade_max_age: Duration,
    pub long_term_max_entries: usize,
    pub confidence_threshold: f64,
    pub enable_flush_worker: bool,
    pub flush_worker_interval: Duration,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            db_path: PathBuf::from("memories.db"),
            write_gate_min_count: 3,
            write_gate_max_age: Duration::from_secs(30 * 60),
            short_term_max_kv: 1000,
            short_term_max_turns: 50,
            short_term_ttl: Duration::from_secs(30 * 60),
            cascade_min_count: 3,
            cascade_max_age: Duration::from_secs(30 * 60),
            long_term_max_entries: 2000,
            confidence_threshold: 0.6,
            enable_flush_worker: true,
            flush_worker_interval: Duration::from_secs(60),
        }
    }
}
