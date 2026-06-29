//! Same-file read content deduplication.
//!
//! When `feature("read_dedup")` is enabled, this module caches file reads
//! keyed by `(path, mtime)`. If the same file is read again within the
//! same session and the mtime hasn't changed, the cached content hash is
//! compared and if it matches, a lightweight "file unchanged" response is
//! returned instead of re-reading the full file contents.
//!
//! This saves ~18% cache tokens by avoiding byte-identical reads.
//! Mirrors Claude Code's `read_dedup_killswitch` logic.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::SystemTime;

/// Cached entry for a single file.
struct CachedEntry {
    /// Modification time at last read.
    mtime: SystemTime,
    /// Quick hash of the content.
    quick_hash: u64,
    /// Number of lines in the file.
    total_lines: usize,
    /// Next offset hint for follow-up reads.
    next_offset: Option<usize>,
}

/// Global read dedup cache, keyed by canonical file path.
static READ_DEDUP_CACHE: Mutex<Option<HashMap<PathBuf, CachedEntry>>> = Mutex::new(None);

/// Compute a quick hash for content dedup using std's DefaultHasher.
fn quick_hash(content: &str) -> u64 {
    use std::hash::{DefaultHasher, Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    // Hash first 8KB for speed
    let head = &content.as_bytes()[..content.len().min(8192)];
    head.hash(&mut hasher);
    // Hash total length to distinguish files with same prefix
    content.len().hash(&mut hasher);
    hasher.finish()
}

/// Result of a dedup cache check.
pub enum DedupCheck {
    /// File is unchanged — return cached metadata.
    Unchanged {
        /// Number of lines in the file (from cache).
        total_lines: usize,
        /// Next offset hint for follow-up reads.
        next_offset: Option<usize>,
    },
    /// File has changed or is not in cache — proceed with full read.
    Changed,
}

/// Check the dedup cache for a file.
pub fn check_dedup(path: &std::path::Path, current_mtime: SystemTime, content: &str) -> DedupCheck {
    let mut guard = READ_DEDUP_CACHE.lock().unwrap();
    let cache = guard.as_mut();
    match cache.and_then(|c| c.get(path)) {
        Some(entry) if entry.mtime == current_mtime && entry.quick_hash == quick_hash(content) => {
            DedupCheck::Unchanged { total_lines: entry.total_lines, next_offset: entry.next_offset }
        }
        _ => DedupCheck::Changed,
    }
}

/// Update the dedup cache after a successful file read.
pub fn update_dedup(
    path: &std::path::Path,
    mtime: SystemTime,
    content: &str,
    total_lines: usize,
    next_offset: Option<usize>,
) {
    let mut guard = READ_DEDUP_CACHE.lock().unwrap();
    let cache = guard.get_or_insert_with(HashMap::new);
    cache.insert(
        path.to_path_buf(),
        CachedEntry { mtime, quick_hash: quick_hash(content), total_lines, next_offset },
    );
}

/// Clear the dedup cache (e.g., on session reset).
pub fn clear_cache() {
    let mut guard = READ_DEDUP_CACHE.lock().unwrap();
    *guard = None;
}

/// Format the "file unchanged" response for the LLM.
pub fn format_unchanged_response(
    file_path: &str,
    total_lines: usize,
    next_offset: Option<usize>,
) -> String {
    let mut output =
        format!("file_unchanged: true ({file_path})\nfile length: {total_lines} lines");
    if let Some(offset) = next_offset {
        output.push_str(&format!("\nprevious_offset: {offset}"));
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::SystemTime;

    #[test]
    fn test_quick_hash_deterministic() {
        let content = "hello world\nline 2\nline 3";
        let h1 = quick_hash(content);
        let h2 = quick_hash(content);
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_quick_hash_different_content() {
        let h1 = quick_hash("hello world");
        let h2 = quick_hash("goodbye world");
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_cache_hit() {
        clear_cache();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.txt");
        std::fs::write(&path, b"hello").unwrap();

        let mtime = std::fs::metadata(&path).unwrap().modified().unwrap();
        let content = "hello";

        update_dedup(&path, mtime, content, 1, None);

        match check_dedup(&path, mtime, content) {
            DedupCheck::Unchanged { total_lines, .. } => assert_eq!(total_lines, 1),
            DedupCheck::Changed => panic!("Should have been cache hit"),
        }
    }

    #[test]
    fn test_cache_miss_different_content() {
        clear_cache();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.txt");
        std::fs::write(&path, b"hello").unwrap();

        let mtime = std::fs::metadata(&path).unwrap().modified().unwrap();

        update_dedup(&path, mtime, "hello", 1, None);

        match check_dedup(&path, mtime, "world") {
            DedupCheck::Unchanged { .. } => panic!("Should have been cache miss"),
            DedupCheck::Changed => {}
        }
    }

    #[test]
    fn test_unchanged_response_format() {
        let resp = format_unchanged_response("/path/to/file.rs", 100, Some(50));
        assert!(resp.contains("file_unchanged: true"));
        assert!(resp.contains("/path/to/file.rs"));
        assert!(resp.contains("100 lines"));
        assert!(resp.contains("previous_offset: 50"));

        // Without next_offset
        let resp2 = format_unchanged_response("/path/to/file.rs", 100, None);
        assert!(resp2.contains("file_unchanged: true"));
        assert!(!resp2.contains("previous_offset"));
    }
}
