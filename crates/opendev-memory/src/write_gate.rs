use crate::types::WriteGateTier;

pub struct WriteGate;

impl WriteGate {
    pub fn classify(content: &str) -> WriteGateTier {
        let lower = content.to_lowercase();

        let structured = ["[steer]", "## subagent result", "plan approved"];
        if structured.iter().any(|p| lower.contains(p)) {
            return WriteGateTier::StructuredPrefix;
        }

        let noise = ["error", "warning", "traceback", "timeout"];
        if noise.iter().any(|p| lower.contains(p)) {
            return WriteGateTier::TransientNoise;
        }

        let register = ["decision", "决定", "convention"];
        if register.iter().any(|p| lower.contains(p)) {
            return WriteGateTier::Register;
        }

        let daily = ["## session summary", "harness insight:"];
        if daily.iter().any(|p| lower.contains(p)) {
            return WriteGateTier::Daily;
        }

        WriteGateTier::Working
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_structured_prefix() {
        assert_eq!(WriteGate::classify("[STEER] use cargo fmt"), WriteGateTier::StructuredPrefix);
    }

    #[test]
    fn classify_transient_noise() {
        assert_eq!(
            WriteGate::classify("timeout while running tests"),
            WriteGateTier::TransientNoise
        );
    }

    #[test]
    fn classify_register() {
        assert_eq!(WriteGate::classify("User decision: use 4 spaces"), WriteGateTier::Register);
    }

    #[test]
    fn classify_daily() {
        assert_eq!(
            WriteGate::classify("## Session Summary\nWe fixed the bug."),
            WriteGateTier::Daily
        );
    }

    #[test]
    fn classify_working_default() {
        assert_eq!(
            WriteGate::classify("The refactor touched three files."),
            WriteGateTier::Working
        );
    }
}
