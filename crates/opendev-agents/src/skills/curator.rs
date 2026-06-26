use chrono::{DateTime, Utc};

use super::metadata::{SkillMetadata, SkillStatus};

const STALE_AFTER_DAYS: i64 = 30;
const ARCHIVE_AFTER_DAYS: i64 = 90;

pub struct Curator;

impl Curator {
    /// Evaluate a skill's lifecycle status based on last_used.
    ///
    /// - Active → Stale after 30 days without use
    /// - Stale → Archived after 90 days without use
    /// - Pinned skills are never changed
    pub fn curate(skill: &mut SkillMetadata, now: DateTime<Utc>) {
        if skill.pinned {
            return;
        }

        let days_since_use = skill.last_used.map(|t| (now - t).num_days()).unwrap_or(0);

        match skill.status {
            SkillStatus::Active => {
                if days_since_use >= STALE_AFTER_DAYS {
                    skill.status = SkillStatus::Stale;
                }
            }
            SkillStatus::Stale => {
                if days_since_use >= ARCHIVE_AFTER_DAYS {
                    skill.status = SkillStatus::Archived;
                }
            }
            SkillStatus::Archived | SkillStatus::Superseded => {}
        }
    }

    /// Reactivate a skill on use.
    pub fn record_usage(skill: &mut SkillMetadata, now: DateTime<Utc>) {
        skill.usage_count += 1;
        skill.last_used = Some(now);
        skill.status = SkillStatus::Active;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn active_skill() -> SkillMetadata {
        SkillMetadata {
            name: "test".into(),
            description: "test skill".into(),
            namespace: "default".into(),
            path: None,
            source: super::super::metadata::SkillSource::Builtin,
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
    fn active_skill_stales_after_30_days() {
        let now = DateTime::UNIX_EPOCH + chrono::Duration::days(100);
        let mut skill = active_skill();
        skill.last_used = Some(DateTime::UNIX_EPOCH);

        Curator::curate(&mut skill, now);
        assert_eq!(skill.status, SkillStatus::Stale);
    }

    #[test]
    fn recent_use_keeps_active() {
        let now = DateTime::UNIX_EPOCH + chrono::Duration::days(100);
        let mut skill = active_skill();
        skill.last_used = Some(now - chrono::Duration::days(5));

        Curator::curate(&mut skill, now);
        assert_eq!(skill.status, SkillStatus::Active);
    }

    #[test]
    fn stale_archives_after_90_days() {
        let mut skill = active_skill();
        skill.status = SkillStatus::Stale;
        skill.last_used = Some(DateTime::UNIX_EPOCH);

        let now = DateTime::UNIX_EPOCH + chrono::Duration::days(95);
        Curator::curate(&mut skill, now);
        assert_eq!(skill.status, SkillStatus::Archived);
    }

    #[test]
    fn pinned_skills_never_change() {
        let mut skill = active_skill();
        skill.pinned = true;
        skill.last_used = Some(DateTime::UNIX_EPOCH);

        let now = DateTime::UNIX_EPOCH + chrono::Duration::days(200);
        Curator::curate(&mut skill, now);
        assert_eq!(skill.status, SkillStatus::Active);
    }

    #[test]
    fn record_usage_reactivates() {
        let mut skill = active_skill();
        skill.status = SkillStatus::Stale;
        skill.last_used = Some(DateTime::UNIX_EPOCH);

        let now = DateTime::UNIX_EPOCH + chrono::Duration::days(50);
        Curator::record_usage(&mut skill, now);
        assert_eq!(skill.status, SkillStatus::Active);
        assert_eq!(skill.usage_count, 1);
        assert_eq!(skill.last_used, Some(now));
    }
}
