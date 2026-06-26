use super::metadata::{SkillMetadata, SkillStatus};

/// Select skills that fit within a token budget.
///
/// Order: pinned skills first, then by usage_count descending.
/// Stale skills are included but deprioritized.
pub fn skills_within_budget<'a>(
    skills: &'a [SkillMetadata],
    token_budget: usize,
) -> Vec<&'a SkillMetadata> {
    let mut injectable: Vec<&SkillMetadata> = skills
        .iter()
        .filter(|s| matches!(s.status, SkillStatus::Active | SkillStatus::Stale))
        .collect();

    injectable.sort_by(|a, b| {
        b.pinned
            .cmp(&a.pinned)
            .then_with(|| b.usage_count.cmp(&a.usage_count))
            .then_with(|| a.name.cmp(&b.name))
    });

    let mut result = Vec::new();
    let mut used = 0usize;
    for skill in injectable {
        let cost = skill.estimate_tokens().unwrap_or(0);
        if used + cost > token_budget {
            continue;
        }
        used += cost;
        result.push(skill);
        if used >= token_budget {
            break;
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skills::metadata::SkillSource;

    fn skill(name: &str, pinned: bool, usage: u32) -> SkillMetadata {
        SkillMetadata {
            name: name.into(),
            description: String::new(),
            namespace: "default".into(),
            path: None,
            source: SkillSource::Builtin,
            model: None,
            agent: None,
            pinned,
            status: SkillStatus::Active,
            requires_tools: None,
            fallback_for_tools: None,
            allowed_tools: None,
            usage_count: usage,
            last_used: None,
            tags: vec![],
        }
    }

    #[test]
    fn pinned_skills_come_first() {
        let skills = vec![skill("b", false, 10), skill("a", true, 0)];
        let result = skills_within_budget(&skills, 9999);
        assert_eq!(result[0].name, "a");
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn sorts_by_usage_desc() {
        let skills = vec![skill("low", false, 1), skill("high", false, 100)];
        let result = skills_within_budget(&skills, 9999);
        assert_eq!(result[0].name, "high");
    }

    #[test]
    fn respects_token_budget() {
        let mut a = skill("big", false, 1);
        let mut b = skill("small", false, 1);
        // estimate_tokens returns content.len() / 4; content is empty for path=None
        // So both are 0 tokens. Let's create skills that have content:
        let content = "a".repeat(400); // ~100 tokens
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("big.md");
        std::fs::write(&path, &content).unwrap();
        a.path = Some(path);
        let path2 = dir.path().join("small.md");
        std::fs::write(&path2, "hello").unwrap(); // ~1 token
        b.path = Some(path2);

        let skills = vec![a, b];
        // Budget of 50 tokens should only fit the small skill
        let result = skills_within_budget(&skills, 50);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "small");
    }
}
