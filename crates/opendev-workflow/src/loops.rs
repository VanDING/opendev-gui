use std::collections::HashSet;

use crate::types::LoopConfig;

#[derive(Debug)]
pub struct LoopState {
    pub results: Vec<String>,
    pub seen: HashSet<String>,
    pub dry_rounds: u32,
    pub rounds: u32,
}

impl LoopState {
    pub fn new() -> Self {
        Self { results: Vec::new(), seen: HashSet::new(), dry_rounds: 0, rounds: 0 }
    }

    pub fn add(&mut self, result: String) -> bool {
        self.rounds += 1;
        if self.seen.insert(result.clone()) {
            self.results.push(result);
            self.dry_rounds = 0;
            true
        } else {
            self.dry_rounds += 1;
            false
        }
    }

    pub fn should_continue(&self, config: &LoopConfig) -> bool {
        if self.rounds >= config.max_iterations {
            return false;
        }
        match config.r#type.as_str() {
            "until_count" => self.results.len() < config.target_count as usize,
            "until_dry" => self.dry_rounds < config.max_dry_rounds,
            "until_budget" => true,
            _ => false,
        }
    }
}

impl Default for LoopState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loop_until_count_stops_at_target() {
        let mut state = LoopState::new();
        let config = LoopConfig {
            r#type: "until_count".into(),
            max_dry_rounds: 3,
            max_iterations: 100,
            target_count: 3,
        };
        state.add("a".into());
        assert!(state.should_continue(&config));
        state.add("b".into());
        assert!(state.should_continue(&config));
        state.add("c".into());
        assert!(!state.should_continue(&config));
    }

    #[test]
    fn loop_until_dry_stops_after_k_empty() {
        let mut state = LoopState::new();
        let config = LoopConfig {
            r#type: "until_dry".into(),
            max_dry_rounds: 3,
            max_iterations: 100,
            target_count: 0,
        };
        state.add("a".into());
        assert!(state.should_continue(&config));
        state.add("a".into()); // duplicate - dry round
        assert!(state.should_continue(&config));
        state.add("a".into()); // duplicate - dry round 2
        assert!(state.should_continue(&config));
        state.add("a".into()); // duplicate - dry round 3, hits max
        assert!(!state.should_continue(&config));
    }
}
