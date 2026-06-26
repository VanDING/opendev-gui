use std::time::{Duration, Instant};

use crate::types::{MemoryCategory, MemorySource, WriteGateTier};

const DEFAULT_MIN_COUNT: usize = 3;
const DEFAULT_MAX_AGE: Duration = Duration::from_secs(30 * 60);

#[derive(Debug, Clone)]
pub struct PendingMemory {
    pub content: String,
    pub category: MemoryCategory,
    pub source: MemorySource,
    pub project_path: Option<std::path::PathBuf>,
    pub importance: f64,
    pub confidence: f64,
    pub staged_at: Instant,
    pub tier: WriteGateTier,
}

#[derive(Debug, Clone)]
pub struct CascadeBuffer {
    entries: Vec<PendingMemory>,
    min_count: usize,
    max_age: Duration,
    created_at: Instant,
}

impl Default for CascadeBuffer {
    fn default() -> Self {
        Self::new(DEFAULT_MIN_COUNT, DEFAULT_MAX_AGE)
    }
}

impl CascadeBuffer {
    pub fn new(min_count: usize, max_age: Duration) -> Self {
        Self { entries: Vec::new(), min_count, max_age, created_at: Instant::now() }
    }

    pub fn stage(&mut self, entry: PendingMemory) {
        self.entries.push(entry);
    }

    pub fn should_flush(&self) -> bool {
        if self.entries.len() >= self.min_count {
            return true;
        }
        if let Some(oldest) = self.entries.iter().map(|e| e.staged_at).min() {
            Instant::now().duration_since(oldest) >= self.max_age
        } else {
            false
        }
    }

    pub fn flush(&mut self) -> Vec<PendingMemory> {
        self.created_at = Instant::now();
        std::mem::take(&mut self.entries)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pending(content: &str) -> PendingMemory {
        PendingMemory {
            content: content.into(),
            category: MemoryCategory::TechnicalNote,
            source: MemorySource::Agent,
            project_path: None,
            importance: 0.5,
            confidence: 0.8,
            staged_at: Instant::now(),
            tier: WriteGateTier::Working,
        }
    }

    #[test]
    fn flush_on_min_count() {
        let mut buf = CascadeBuffer::new(3, Duration::from_secs(60));
        for i in 0..2 {
            buf.stage(pending(&format!("{i}")));
        }
        assert!(!buf.should_flush());
        buf.stage(pending("2"));
        assert!(buf.should_flush());
        let flushed = buf.flush();
        assert_eq!(flushed.len(), 3);
        assert!(buf.is_empty());
    }

    #[test]
    fn flush_on_age() {
        let mut buf = CascadeBuffer::new(10, Duration::from_millis(1));
        buf.stage(pending("old"));
        std::thread::sleep(Duration::from_millis(5));
        assert!(buf.should_flush());
    }
}
