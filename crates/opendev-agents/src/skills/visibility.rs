use std::collections::HashSet;

use super::metadata::{SkillMetadata, SkillStatus};

/// Check whether a skill should be visible to the agent given the available tools.
pub fn is_visible(skill: &SkillMetadata, available_tools: &HashSet<String>) -> bool {
    // Archived and superseded skills are invisible
    if !matches!(skill.status, SkillStatus::Active | SkillStatus::Stale) {
        return false;
    }

    // If the skill requires specific tools, all must be available
    if let Some(required) = &skill.requires_tools {
        if !required.iter().all(|t| available_tools.contains(t)) {
            return false;
        }
    }

    true
}

/// Return the fallback skills for a missing tool.
pub fn fallback_skills<'a>(
    skills: &'a [SkillMetadata],
    missing_tool: &str,
) -> Vec<&'a SkillMetadata> {
    skills
        .iter()
        .filter(|s| {
            s.status == SkillStatus::Active
                && s.fallback_for_tools
                    .as_ref()
                    .is_some_and(|tools| tools.iter().any(|t| t == missing_tool))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_skill(name: &str, status: SkillStatus) -> SkillMetadata {
        SkillMetadata {
            name: name.into(),
            description: String::new(),
            namespace: "default".into(),
            path: None,
            source: SkillSource::Builtin,
            model: None,
            agent: None,
            pinned: false,
            status,
            requires_tools: None,
            fallback_for_tools: None,
            allowed_tools: None,
            usage_count: 0,
            last_used: None,
            tags: vec![],
        }
    }

    #[test]
    fn active_skill_is_visible() {
        let tools = ["read", "write"].into_iter().map(String::from).collect();
        let skill = make_skill("test", SkillStatus::Active);
        assert!(is_visible(&skill, &tools));
    }

    #[test]
    fn archived_skill_is_invisible() {
        let tools = ["read"].into_iter().map(String::from).collect();
        let skill = make_skill("old", SkillStatus::Archived);
        assert!(!is_visible(&skill, &tools));
    }

    #[test]
    fn skill_with_missing_required_tool_is_invisible() {
        let tools = ["read"].into_iter().map(String::from).collect();
        let mut skill = make_skill("test", SkillStatus::Active);
        skill.requires_tools = Some(vec!["write".into(), "read".into()]);
        assert!(!is_visible(&skill, &tools));
    }

    #[test]
    fn skill_with_all_required_tools_is_visible() {
        let tools = ["read", "write"].into_iter().map(String::from).collect();
        let mut skill = make_skill("test", SkillStatus::Active);
        skill.requires_tools = Some(vec!["read".into()]);
        assert!(is_visible(&skill, &tools));
    }

    #[test]
    fn fallback_for_missing_tool() {
        let mut s1 = make_skill("editor", SkillStatus::Active);
        s1.fallback_for_tools = Some(vec!["write".into()]);
        let s2 = make_skill("other", SkillStatus::Active);

        let skills = vec![s1, s2];
        let result = fallback_skills(&skills, "write");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "editor");
    }
}
