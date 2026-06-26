use super::super::metadata::{SkillMetadata, SkillStatus};
use super::*;

// ---- URL fetching ----

#[test]
fn test_fetch_url_invalid_command() {
    // Unreachable URL should return error
    let result = fetch_url("https://192.0.2.1/nonexistent");
    assert!(result.is_err());
}

#[test]
fn test_pull_url_skills_invalid_url() {
    let result = pull_url_skills("https://192.0.2.1/nonexistent");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("curl failed"));
}

#[test]
fn test_skill_source_url_display() {
    let source = SkillSource::Url("https://example.com/skills".to_string());
    assert_eq!(source.to_string(), "url:https://example.com/skills");
}

// ---- Cache invalidation via mtime ----

fn base_meta(name: &str, desc: &str) -> SkillMetadata {
    SkillMetadata {
        name: name.to_string(),
        description: desc.to_string(),
        namespace: "default".to_string(),
        path: None,
        source: SkillSource::Builtin,
        model: None,
        agent: None,
        pinned: false,
        status: SkillStatus::Active,
        requires_tools: None,
        fallback_for_tools: None,
        allowed_tools: None,
        usage_count: 0,
        last_used: None,
        tags: vec![],
    }
}

fn project_meta(name: &str, desc: &str, file: std::path::PathBuf) -> SkillMetadata {
    SkillMetadata {
        name: name.to_string(),
        description: desc.to_string(),
        namespace: "default".to_string(),
        path: Some(file),
        source: SkillSource::Project,
        model: None,
        agent: None,
        pinned: false,
        status: SkillStatus::Active,
        requires_tools: None,
        fallback_for_tools: None,
        allowed_tools: None,
        usage_count: 0,
        last_used: None,
        tags: vec![],
    }
}

#[test]
fn test_is_cache_stale_builtin_never_stale() {
    let skill = LoadedSkill {
        metadata: base_meta("commit", "Builtin commit"),
        content: "content".to_string(),
        companion_files: vec![],
        cached_mtime: None,
    };
    assert!(!is_cache_stale(&skill));
}

#[test]
fn test_is_cache_stale_no_mtime_not_stale() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("skill.md");
    std::fs::write(&file, "---\nname: test\ndescription: t\n---\ncontent").unwrap();

    let skill = LoadedSkill {
        metadata: project_meta("test", "t", file),
        content: "content".to_string(),
        companion_files: vec![],
        cached_mtime: None, // No mtime recorded
    };
    assert!(!is_cache_stale(&skill));
}

#[test]
fn test_is_cache_stale_unmodified_file() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("skill.md");
    std::fs::write(&file, "---\nname: test\ndescription: t\n---\ncontent").unwrap();

    let mtime = std::fs::metadata(&file).unwrap().modified().unwrap();

    let skill = LoadedSkill {
        metadata: project_meta("test", "t", file),
        content: "content".to_string(),
        companion_files: vec![],
        cached_mtime: Some(mtime),
    };
    assert!(!is_cache_stale(&skill));
}

#[test]
fn test_is_cache_stale_modified_file() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("skill.md");
    std::fs::write(&file, "---\nname: test\ndescription: t\n---\noriginal").unwrap();

    // Record an old mtime (1 second in the past).
    let old_mtime = std::time::SystemTime::now() - std::time::Duration::from_secs(2);

    let skill = LoadedSkill {
        metadata: project_meta("test", "t", file),
        content: "original".to_string(),
        companion_files: vec![],
        cached_mtime: Some(old_mtime),
    };

    // File was written "now", cached mtime is 2s in the past → stale.
    assert!(is_cache_stale(&skill));
}

#[test]
fn test_is_cache_stale_deleted_file() {
    let skill = LoadedSkill {
        metadata: project_meta("gone", "t", std::path::PathBuf::from("/nonexistent/skill.md")),
        content: "content".to_string(),
        companion_files: vec![],
        cached_mtime: Some(std::time::SystemTime::now()),
    };
    // File doesn't exist → not stale (keep cache).
    assert!(!is_cache_stale(&skill));
}
