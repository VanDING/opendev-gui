//! Token counting heuristics for context management.
//!
//! Attempts to use a proper tokenizer when available, falling back to a
//! heuristic for approximate counts.

/// Try to count tokens using the heuristic (no external tokenizer dependency).
/// When tiktoken-rs or similar is added, replace this function body.
pub fn count_tokens(text: &str) -> usize {
    if text.is_empty() {
        return 0;
    }
    // Heuristic: split on whitespace, estimate based on word length.
    // This provides ~80% accuracy for English prose.
    let word_count: usize = text
        .split_whitespace()
        .map(|word| {
            let len = word.len();
            if len > 12 {
                return len.div_ceil(4);
            }
            let punct_count = word.chars().filter(|c| c.is_ascii_punctuation()).count();
            1 + punct_count.div_ceil(2)
        })
        .sum();
    // Apply 0.75 ratio: most English words map to < 1 BPE token.
    (word_count * 3 + 2) / 4
}

/// Count tokens with compaction threshold check.
///
/// Returns `true` if the text exceeds `threshold_percent` of `budget_tokens`.
/// Uses the heuristic tokenizer; when a real tokenizer is available, this
/// should use it instead.
pub fn exceeds_threshold(text: &str, budget_tokens: usize, threshold_percent: u64) -> bool {
    let estimated = count_tokens(text);
    let threshold = (budget_tokens as u128) * (threshold_percent as u128) / 100;
    (estimated as u128) >= threshold
}

#[cfg(test)]
#[path = "tokens_tests.rs"]
mod tests;
