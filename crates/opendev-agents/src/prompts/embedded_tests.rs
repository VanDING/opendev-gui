use super::*;

#[test]
fn test_all_templates_embedded() {
    assert_eq!(TEMPLATES.len(), TEMPLATE_COUNT);
}

#[test]
fn test_get_embedded_known() {
    let content = get_embedded("system/main/main-security-policy.md");
    assert!(content.is_some());
    assert!(content.unwrap().contains("Security Policy"));
}

#[test]
fn test_get_embedded_unknown() {
    assert!(get_embedded("nonexistent.md").is_none());
}

#[test]
fn test_system_main_templates() {
    let templates = system_main_templates();
    assert!(templates.len() >= 18);
    assert!(templates.iter().all(|(k, _)| k.starts_with("system/main/")));
}

#[test]
fn test_tool_templates() {
    let templates = tool_templates();
    assert!(templates.len() >= 30);
    assert!(templates.iter().all(|(k, _)| k.starts_with("tools/")));
}

#[test]
fn test_subagent_templates() {
    let templates = subagent_templates();
    assert!(templates.len() >= 4);
}

#[test]
fn test_build_init_prompt_no_args() {
    let prompt = build_init_prompt("");
    assert!(prompt.contains("AGENTS.md"));
    assert!(prompt.contains("Build/lint/test"));
    assert!(!prompt.contains("{args}"));
}

#[test]
fn test_build_init_prompt_with_args() {
    let prompt = build_init_prompt("focus on testing");
    assert!(prompt.contains("focus on testing"));
    assert!(!prompt.contains("{args}"));
}

#[test]
fn test_no_empty_templates() {
    for (path, content) in TEMPLATES.iter() {
        assert!(!content.is_empty(), "Template {} is empty", path);
    }
}

/// Verify that all section file paths registered in `create_default_composer`
/// have corresponding entries in the embedded template registry.
///
/// This ensures that every prompt section referenced by the factory can
/// be resolved from embedded templates (no missing template files).
#[test]
fn test_all_factory_sections_have_embedded_templates() {
    use crate::prompts::composer::create_default_composer;

    let dir = tempfile::TempDir::new().unwrap();
    let composer = create_default_composer(dir.path());

    let names: Vec<String> = composer.section_names().iter().map(|s| s.to_string()).collect();

    // Verify TEMPLATE_COUNT matches the actual number of embedded templates.
    assert_eq!(
        TEMPLATES.len(),
        TEMPLATE_COUNT,
        "TEMPLATE_COUNT constant ({}) does not match actual embedded template count ({})",
        TEMPLATE_COUNT,
        TEMPLATES.len(),
    );

    // Verifies all 22+ system/main sections in the factory are embedded
    let main_templates = system_main_templates();
    let section_count = names.len();
    assert!(
        section_count >= 22,
        "Expected at least 22 registered sections, got {}",
        section_count,
    );
    assert!(
        main_templates.len() >= 22,
        "Expected at least 22 system/main embedded templates, got {}",
        main_templates.len(),
    );
}

/// Verify that specific key template files mentioned in the design
/// documentation have corresponding entries.
#[test]
fn test_known_templates_exist() {
    let required = [
        "system/main/main-provider-openai.md",
        "system/main/main-provider-anthropic.md",
        "system/main/main-provider-fireworks.md",
        "system/main/main-code-references.md",
        "system/main/main-output-awareness.md",
        "system/main/main-no-time-estimates.md",
        "system/main/main-mode-awareness.md",
        "system/main/main-security-policy.md",
        "system/main/main-tone-and-style.md",
        "system/main/main-output-efficiency.md",
    ];

    for &path in &required {
        assert!(
            TEMPLATES.contains_key(path),
            "Required embedded template '{}' is missing from TEMPLATES registry",
            path,
        );
        let content = get_embedded(path);
        assert!(
            content.is_some(),
            "Required template '{}' not found via get_embedded()",
            path,
        );
        assert!(
            !content.unwrap().is_empty(),
            "Required template '{}' is empty",
            path,
        );
    }
}
