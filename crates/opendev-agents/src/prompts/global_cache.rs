//! Global prompt cache — stores rendered prompt sections on disk keyed by
//! SHA-256 content hash, avoiding repeated re-rendering of static sections.
//!
//! Cache entries are invalidated on:
//! - Version change (OpenDev release)
//! - Config change
//! - 24-hour TTL expiry

use std::path::{Path, PathBuf};

/// Cache entry TTL (24 hours).
const CACHE_TTL_SECS: u64 = 24 * 60 * 60;

/// Subdirectory for prompt cache under `~/.opendev/`.
const CACHE_SUBDIR: &str = "prompt-cache";

/// Global prompt cache backed by the filesystem.
///
/// Rendered prompt content is stored as `{hash}.txt` files under
/// `~/.opendev/prompt-cache/` and invalidated when the content hash,
/// version, or config changes.
#[derive(Debug)]
pub struct GlobalPromptCache {
    cache_dir: PathBuf,
    /// Current version string (invalidates cache on version change).
    version: String,
    /// Current config hash (invalidates cache on config change).
    config_hash: String,
}

impl GlobalPromptCache {
    /// Create a new prompt cache.
    ///
    /// `version` should be a unique identifier for the current OpenDev
    /// version (e.g., `"0.1.9"`). `config_hash` should change whenever
    /// user configuration that affects prompts changes.
    pub fn new(version: impl Into<String>, config_hash: impl Into<String>) -> Self {
        let base = dirs_next().unwrap_or_else(|| PathBuf::from("."));
        let cache_dir = base.join(CACHE_SUBDIR);
        let _ = std::fs::create_dir_all(&cache_dir);
        Self { cache_dir, version: version.into(), config_hash: config_hash.into() }
    }

    /// Compute the SHA-256 hash of content.
    pub fn hash_content(&self, content: &str) -> String {
        use sha2::Digest;
        let mut hasher = sha2::Sha256::new();
        // Mix version and config hash into the content hash so that
        // version or config changes automatically invalidate the cache.
        hasher.update(self.version.as_bytes());
        hasher.update(b":");
        hasher.update(self.config_hash.as_bytes());
        hasher.update(b":");
        hasher.update(content.as_bytes());
        let result = hasher.finalize();
        hex::encode(result)
    }

    /// Try to load cached content by hash.
    ///
    /// Returns `None` if the cache entry doesn't exist or is expired.
    pub fn load(&self, hash: &str) -> Option<String> {
        let path = self.cache_dir.join(format!("{hash}.txt"));
        if !path.exists() {
            return None;
        }
        // Check TTL
        if let Ok(metadata) = std::fs::metadata(&path) {
            if let Ok(modified) = metadata.modified() {
                if let Ok(age) = modified.elapsed() {
                    if age.as_secs() > CACHE_TTL_SECS {
                        // Expired — remove and return None
                        let _ = std::fs::remove_file(&path);
                        return None;
                    }
                }
            }
        }
        std::fs::read_to_string(&path).ok()
    }

    /// Store content in the cache.
    pub fn store(&self, hash: &str, content: &str) {
        let path = self.cache_dir.join(format!("{hash}.txt"));
        let _ = std::fs::create_dir_all(&self.cache_dir);
        let _ = std::fs::write(&path, content);
    }

    /// Invalidate all cached entries (clear the cache directory).
    pub fn invalidate_all(&self) {
        if let Ok(entries) = std::fs::read_dir(&self.cache_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |e| e == "txt") {
                    let _ = std::fs::remove_file(&path);
                }
            }
        }
    }

    /// Compute hash and load/store in one call.
    ///
    /// If the content is already cached, returns the cached version.
    /// Otherwise, renders via `render_fn`, caches the result, and returns it.
    pub fn get_or_compute(&self, content: &str, render_fn: impl FnOnce(&str) -> String) -> String {
        let hash = self.hash_content(content);
        if let Some(cached) = self.load(&hash) {
            return cached;
        }
        let rendered = render_fn(content);
        self.store(&hash, &rendered);
        rendered
    }
}

/// Get the OpenDev data directory (`~/.opendev/`).
fn dirs_next() -> Option<PathBuf> {
    dirs::home_dir().map(|d| d.join(".opendev"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_content_deterministic() {
        let cache = GlobalPromptCache::new("1.0", "default");
        let h1 = cache.hash_content("hello world");
        let h2 = cache.hash_content("hello world");
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_hash_content_differs() {
        let cache = GlobalPromptCache::new("1.0", "default");
        let h1 = cache.hash_content("hello");
        let h2 = cache.hash_content("world");
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_hash_changes_with_version() {
        let cache1 = GlobalPromptCache::new("1.0", "default");
        let cache2 = GlobalPromptCache::new("2.0", "default");
        assert_ne!(cache1.hash_content("same content"), cache2.hash_content("same content"));
    }

    #[test]
    fn test_store_and_load() {
        let dir = tempfile::tempdir().unwrap();
        let cache_dir = dir.path().join("prompt-cache");
        let base = dir.path().to_path_buf();

        // Override dirs_next by setting up manually
        let cache = GlobalPromptCache {
            cache_dir,
            version: "test".to_string(),
            config_hash: "test".to_string(),
        };

        let hash = cache.hash_content("stored content");
        cache.store(&hash, "stored content");
        let loaded = cache.load(&hash);
        assert_eq!(loaded, Some("stored content".to_string()));
    }

    #[test]
    fn test_get_or_compute_caches() {
        let dir = tempfile::tempdir().unwrap();
        let cache = GlobalPromptCache {
            cache_dir: dir.path().join("prompt-cache"),
            version: "test".to_string(),
            config_hash: "test".to_string(),
        };

        let mut call_count = 0;
        let result = cache.get_or_compute("input", |s| {
            call_count += 1;
            format!("rendered: {s}")
        });
        assert_eq!(result, "rendered: input");
        assert_eq!(call_count, 1);

        // Second call should use cache
        let result2 = cache.get_or_compute("input", |_| {
            call_count += 1;
            "should not be called".to_string()
        });
        assert_eq!(result2, "rendered: input");
        assert_eq!(call_count, 1);
    }
}
