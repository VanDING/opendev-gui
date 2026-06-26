use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

const DEFAULT_MAX_KV: usize = 1000;
const DEFAULT_MAX_TURNS: usize = 50;
const DEFAULT_TTL: Duration = Duration::from_secs(30 * 60);

#[derive(Debug, Clone)]
pub struct ShortTermMemory {
    kv: HashMap<String, (String, Instant)>,
    turns: VecDeque<String>,
    max_kv: usize,
    max_turns: usize,
    ttl: Duration,
}

impl Default for ShortTermMemory {
    fn default() -> Self {
        Self::new(DEFAULT_MAX_KV, DEFAULT_MAX_TURNS, DEFAULT_TTL)
    }
}

impl ShortTermMemory {
    pub fn new(max_kv: usize, max_turns: usize, ttl: Duration) -> Self {
        Self { kv: HashMap::new(), turns: VecDeque::new(), max_kv, max_turns, ttl }
    }

    pub fn set(&mut self, key: String, value: String) {
        self.kv.insert(key, (value, Instant::now()));
        self.prune();
    }

    pub fn get(&mut self, key: &str) -> Option<&str> {
        self.prune();
        self.kv.get(key).map(|(v, _)| v.as_str())
    }

    pub fn push_turn(&mut self, log: String) {
        self.turns.push_back(log);
        while self.turns.len() > self.max_turns {
            self.turns.pop_front();
        }
    }

    pub fn recent_turns(&self, limit: usize) -> Vec<&str> {
        self.turns.iter().rev().take(limit).map(|s| s.as_str()).collect()
    }

    pub fn prune(&mut self) {
        let now = Instant::now();
        self.kv.retain(|_, (_, t)| now.duration_since(*t) < self.ttl);
        while self.kv.len() > self.max_kv {
            if let Some(oldest) =
                self.kv.iter().min_by_key(|(_, (_, t))| *t).map(|(k, _)| k.clone())
            {
                self.kv.remove(&oldest);
            } else {
                break;
            }
        }
    }

    pub fn kv_len(&self) -> usize {
        self.kv.len()
    }

    pub fn turn_count(&self) -> usize {
        self.turns.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_and_get() {
        let mut stm = ShortTermMemory::default();
        stm.set("key".into(), "value".into());
        assert_eq!(stm.get("key"), Some("value"));
    }

    #[test]
    fn expires_after_ttl() {
        let mut stm = ShortTermMemory::new(10, 10, Duration::from_millis(1));
        stm.set("key".into(), "value".into());
        std::thread::sleep(Duration::from_millis(10));
        assert_eq!(stm.get("key"), None);
    }

    #[test]
    fn turn_buffer_limits() {
        let mut stm = ShortTermMemory::default();
        for i in 0..60 {
            stm.push_turn(format!("turn {i}"));
        }
        assert_eq!(stm.turn_count(), 50);
        assert_eq!(stm.recent_turns(3), vec!["turn 59", "turn 58", "turn 57"]);
    }
}
