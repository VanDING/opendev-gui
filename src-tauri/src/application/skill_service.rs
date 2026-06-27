//! SkillService — Skill discovery and pin management.

use opendev_agents::{SkillLoader, SkillMetadata};

/// Serializable skill info for frontend display.
#[derive(Debug, Clone, serde::Serialize)]
pub struct SkillInfo {
    pub name: String,
    pub description: String,
    pub namespace: String,
    pub source: String,
    pub pinned: bool,
    pub status: String,
    pub usage_count: u32,
    pub tags: Vec<String>,
}

impl From<&SkillMetadata> for SkillInfo {
    fn from(meta: &SkillMetadata) -> Self {
        SkillInfo {
            name: meta.name.clone(),
            description: meta.description.clone(),
            namespace: meta.namespace.clone(),
            source: meta.source.to_string(),
            pinned: meta.pinned,
            status: format!("{:?}", meta.status),
            usage_count: meta.usage_count,
            tags: meta.tags.clone(),
        }
    }
}

pub struct SkillService {
    skill_loader: std::sync::Mutex<Option<SkillLoader>>,
}

impl SkillService {
    pub fn new() -> Self {
        Self {
            skill_loader: std::sync::Mutex::new(None),
        }
    }

    /// Set the skill loader (called once during initialization).
    pub fn set_skill_loader(&self, loader: SkillLoader) {
        *self.skill_loader.lock().unwrap() = Some(loader);
    }

    /// List all discovered skills.
    pub fn list_skills(&self) -> Vec<SkillInfo> {
        let mut guard = self.skill_loader.lock().unwrap();
        match guard.as_mut() {
            Some(loader) => {
                let skills = loader.discover_skills();
                skills.iter().map(SkillInfo::from).collect()
            }
            None => Vec::new(),
        }
    }

    /// Toggle the pinned status of a skill. Returns the new pinned state, or an error.
    pub fn toggle_pin(&self, name: &str) -> Result<bool, String> {
        let mut guard = self.skill_loader.lock().unwrap();
        let loader = guard.as_mut().ok_or("Skills system not initialized")?;

        let skills = loader.discover_skills();
        let found = skills
            .iter()
            .find(|s| s.full_name() == name || s.name == name)
            .ok_or_else(|| format!("Skill '{}' not found", name))?;

        let new_pinned = !found.pinned;
        loader.set_pinned(&found.full_name(), new_pinned);
        Ok(new_pinned)
    }
}
