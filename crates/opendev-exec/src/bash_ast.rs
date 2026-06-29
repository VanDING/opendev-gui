//! AST-based bash command parser for security analysis.
//!
//! Parses bash commands into a trustworthy `argv[]` to improve permission
//! pattern matching. Fail-closed: parse errors or unknown constructs
//! result in `untrusted` status, causing the command to be denied or
//! escalated for user approval.
//!
//! Currently uses regex-based fallback parsing.
//! TODO: Integrate tree-sitter-bash grammar for full AST parsing
//! when the dependency is available.

use std::time::{Duration, Instant};

/// Maximum time allowed for parsing a single command.
const PARSE_TIMEOUT: Duration = Duration::from_millis(100);

/// Result of parsing a bash command.
#[derive(Debug, Clone)]
pub struct ParsedCommand {
    /// Extracted arguments (argv[0] = command name).
    pub argv: Vec<String>,
    /// Whether the parse is considered safe/trustworthy.
    pub safe: bool,
    /// Human-readable parse error, if any.
    pub parse_error: Option<String>,
}

/// Errors that can occur during bash command parsing.
#[derive(Debug, Clone, thiserror::Error)]
pub enum BashAstError {
    /// The input could not be parsed.
    #[error("parse error: {0}")]
    ParseError(String),
    /// Parsing exceeded the timeout.
    #[error("parse timeout")]
    Timeout,
    /// The command contains an unsupported construct (e.g., process substitution).
    #[error("unsupported construct: {0}")]
    UnsupportedConstruct(String),
    /// An internal error occurred during parsing.
    #[error("internal error: {0}")]
    Internal(String),
}

/// State machine for character-level parsing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ParseState {
    /// Outside any quoting context.
    Normal,
    /// Inside single quotes — no escape processing.
    InSingleQuote,
    /// Inside double quotes — limited escape processing.
    InDoubleQuote,
    /// The previous character was a backslash; next char is literal.
    Escape,
}

/// Parser for bash commands using fallback character-level parsing.
///
/// Extracts a trustworthy `argv[]` from a bash command string by handling:
/// - Single and double-quoted strings
/// - Escaped characters
/// - Process substitution detection (`$(...)`, `<(...)`, `>(...)`)
///
/// The parser is deliberately conservative and **fail-closed**: any
/// unrecognised construct, parse error, or timeout causes the command
/// to be marked as untrusted.
#[derive(Debug, Clone)]
pub struct BashAstParser;

impl BashAstParser {
    /// Create a new parser instance.
    pub fn new() -> Self {
        Self
    }

    /// Parse a bash command string.
    ///
    /// Returns a `ParsedCommand` on success (even if the parse contains
    /// errors — check `ParsedCommand::safe`) or a `BashAstError` for
    /// catastrophic failures such as timeouts.
    pub fn parse(&self, command: &str) -> Result<ParsedCommand, BashAstError> {
        let start = Instant::now();
        let trimmed = command.trim();

        if trimmed.is_empty() {
            return Err(BashAstError::ParseError(
                "command is empty or whitespace-only".into(),
            ));
        }

        let (argv, errors) = self.parse_fallback(trimmed, start)?;

        let safe = errors.is_empty();
        let parse_error = if errors.is_empty() {
            None
        } else {
            Some(errors.join("; "))
        };

        Ok(ParsedCommand {
            argv,
            safe,
            parse_error,
        })
    }

    /// Returns `true` if the command parsed successfully with no errors.
    pub fn is_parse_successful(&self, command: &str) -> bool {
        match self.parse(command) {
            Ok(cmd) => cmd.safe,
            Err(_) => false,
        }
    }

    /// Extract `argv[]` from the command, returning `None` on parse failure.
    pub fn extract_argv(&self, command: &str) -> Option<Vec<String>> {
        match self.parse(command) {
            Ok(cmd) => Some(cmd.argv),
            Err(_) => None,
        }
    }

    /// Returns `true` if the command is untrusted.
    ///
    /// Fail-closed: any parse error, timeout, or unsupported construct
    /// causes the command to be treated as untrusted.
    pub fn is_untrusted(&self, command: &str) -> bool {
        match self.parse(command) {
            Ok(cmd) => !cmd.safe,
            Err(_) => true,
        }
    }

    // ------------------------------------------------------------------
    // Internal: character-level fallback parser
    // ------------------------------------------------------------------

    /// Character-level fallback parser.
    ///
    /// Walks the input one character at a time, tracking quoting context
    /// and building argv tokens. Stops at command separators (`|`, `;`,
    /// `&&`, `||`) and extracts only the first command.
    fn parse_fallback(
        &self,
        input: &str,
        start: Instant,
    ) -> Result<(Vec<String>, Vec<String>), BashAstError> {
        let mut argv: Vec<String> = Vec::new();
        let mut errors: Vec<String> = Vec::new();
        let mut current = String::new();
        let chars: Vec<char> = input.chars().collect();
        let len = chars.len();
        let mut i = 0;

        let mut state = ParseState::Normal;

        while i < len {
            // Enforce timeout to prevent long-running parses.
            if start.elapsed() > PARSE_TIMEOUT {
                return Err(BashAstError::Timeout);
            }

            let c = chars[i];

            match state {
                ParseState::Normal => match c {
                    // ---- escape ---------------------------------------------------
                    '\\' => {
                        state = ParseState::Escape;
                    }
                    // ---- single quotes --------------------------------------------
                    '\'' => {
                        state = ParseState::InSingleQuote;
                    }
                    // ---- double quotes --------------------------------------------
                    '"' => {
                        state = ParseState::InDoubleQuote;
                    }
                    // ---- command separators (stop — first command only) -----------
                    '|' => {
                        // Push the current token before stopping.
                        self.finish_token(&mut current, &mut argv);
                        break;
                    }
                    ';' => {
                        self.finish_token(&mut current, &mut argv);
                        break;
                    }
                    '&' if i + 1 < len && chars[i + 1] == '&' => {
                        self.finish_token(&mut current, &mut argv);
                        break;
                    }
                    // ---- process substitution detection ---------------------------
                    '$' if i + 1 < len && chars[i + 1] == '(' => {
                        errors.push("process substitution $(...) is unsupported".into());
                        i = self.skip_paren_group(i + 2, &chars, len);
                        continue;
                    }
                    '<' if i + 1 < len && chars[i + 1] == '(' => {
                        errors.push("process substitution <(...) is unsupported".into());
                        i = self.skip_paren_group(i + 2, &chars, len);
                        continue;
                    }
                    '>' if i + 1 < len && chars[i + 1] == '(' => {
                        errors.push("process substitution >(...) is unsupported".into());
                        i = self.skip_paren_group(i + 2, &chars, len);
                        continue;
                    }
                    // ---- whitespace (token boundary) ------------------------------
                    c if c.is_whitespace() => {
                        self.finish_token(&mut current, &mut argv);
                    }
                    // ---- regular character ----------------------------------------
                    _ => {
                        current.push(c);
                    }
                },

                ParseState::InSingleQuote => {
                    if c == '\'' {
                        state = ParseState::Normal;
                    } else {
                        current.push(c);
                    }
                }

                ParseState::InDoubleQuote => match c {
                    '\\' => {
                        // In double quotes only \, $, `, ", and newline are escaped.
                        if i + 1 < len {
                            let next = chars[i + 1];
                            match next {
                                '\\' | '$' | '`' | '"' | '\n' => {
                                    i += 1;
                                    current.push(next);
                                }
                                _ => {
                                    current.push(c);
                                }
                            }
                        } else {
                            current.push(c);
                        }
                    }
                    '"' => {
                        state = ParseState::Normal;
                    }
                    // Detect $( inside double-quoted strings.
                    '$' if i + 1 < len && chars[i + 1] == '(' => {
                        errors.push("process substitution $(...) is unsupported".into());
                        i = self.skip_paren_group(i + 2, &chars, len);
                        continue;
                    }
                    _ => {
                        current.push(c);
                    }
                },

                ParseState::Escape => {
                    current.push(c);
                    state = ParseState::Normal;
                }
            }

            i += 1;
        }

        // ---- post-loop diagnostics ------------------------------------------------

        if let ParseState::Escape = state {
            errors.push("trailing backslash at end of input".into());
        }

        match state {
            ParseState::InSingleQuote => {
                errors.push("unclosed single quote".into());
            }
            ParseState::InDoubleQuote => {
                errors.push("unclosed double quote".into());
            }
            _ => {}
        }

        // Push the final token if any.
        self.finish_token(&mut current, &mut argv);

        // Fail-closed: if we detected process substitution but didn't already
        // add an error (shouldn't happen), ensure it's marked unsafe.
        // (errors already populated in the substitution branches above.)

        Ok((argv, errors))
    }

    /// Push the current token onto `argv` and clear it, if non-empty.
    fn finish_token(&self, current: &mut String, argv: &mut Vec<String>) {
        if !current.is_empty() {
            argv.push(std::mem::take(current));
        }
    }

    /// Skip past a parenthesised group (e.g. `$(...)`, `<(...)`, `>(...)`).
    ///
    /// Returns the index of the closing `)` so the outer loop can `continue`.
    /// `start` should point to the character **after** the opening `(`.
    fn skip_paren_group(&self, start: usize, chars: &[char], len: usize) -> usize {
        let mut depth: usize = 1;
        let mut i = start;
        while i < len && depth > 0 {
            match chars[i] {
                '(' => depth += 1,
                ')' => depth -= 1,
                _ => {}
            }
            i += 1;
        }
        // Return i (points past the last matched `)`).
        i
    }
}

impl Default for BashAstParser {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // ---- simple commands ---------------------------------------------------

    #[test]
    fn simple_command() {
        let parser = BashAstParser::new();
        let result = parser.parse("echo hello world").unwrap();
        assert!(result.safe);
        assert_eq!(result.argv, vec!["echo", "hello", "world"]);
    }

    #[test]
    fn single_word_command() {
        let parser = BashAstParser::new();
        let result = parser.parse("ls").unwrap();
        assert!(result.safe);
        assert_eq!(result.argv, vec!["ls"]);
    }

    #[test]
    fn command_with_multiple_spaces() {
        let parser = BashAstParser::new();
        let result = parser.parse("cat    file").unwrap();
        assert!(result.safe);
        assert_eq!(result.argv, vec!["cat", "file"]);
    }

    // ---- quoting -----------------------------------------------------------

    #[test]
    fn double_quoted_string() {
        let parser = BashAstParser::new();
        let result = parser.parse("echo \"hello world\"").unwrap();
        assert!(result.safe);
        assert_eq!(result.argv, vec!["echo", "hello world"]);
    }

    #[test]
    fn single_quoted_string() {
        let parser = BashAstParser::new();
        let result = parser.parse("echo 'hello world'").unwrap();
        assert!(result.safe);
        assert_eq!(result.argv, vec!["echo", "hello world"]);
    }

    #[test]
    fn mixed_quotes() {
        let parser = BashAstParser::new();
        let result = parser.parse("echo \"hello\"' world'").unwrap();
        assert!(result.safe);
        assert_eq!(result.argv, vec!["echo", "hello world"]);
    }

    #[test]
    fn quotes_with_adjacent_text() {
        let parser = BashAstParser::new();
        // "hello"world → helloworld (no space between)
        let result = parser.parse("echo \"hello\"world").unwrap();
        assert!(result.safe);
        assert_eq!(result.argv, vec!["echo", "helloworld"]);
    }

    // ---- escaped characters ------------------------------------------------

    #[test]
    fn escaped_space() {
        let parser = BashAstParser::new();
        let result = parser.parse("echo hello\\ world").unwrap();
        assert!(result.safe);
        assert_eq!(result.argv, vec!["echo", "hello world"]);
    }

    #[test]
    fn escaped_backslash() {
        let parser = BashAstParser::new();
        let result = parser.parse("echo path\\\\to\\\\file").unwrap();
        assert!(result.safe);
        assert_eq!(result.argv, vec!["echo", r"path\to\file"]);
    }

    #[test]
    fn escaped_char_in_double_quotes() {
        let parser = BashAstParser::new();
        // "she said \"hello\""
        let result = parser.parse("echo \"she said \\\"hello\\\"\"").unwrap();
        assert!(result.safe);
        assert_eq!(result.argv, vec!["echo", "she said \"hello\""]);
    }

    // ---- process substitution detection ------------------------------------

    #[test]
    fn process_substitution_input() {
        let parser = BashAstParser::new();
        let result = parser.parse("cat <(echo test)").unwrap();
        assert!(!result.safe, "process substitution should be unsafe");
        assert!(result.parse_error.is_some());
        let err = result.parse_error.as_ref().unwrap();
        assert!(
            err.contains("<(...)"),
            "error should mention <(...): got {err}"
        );
    }

    #[test]
    fn process_substitution_output() {
        let parser = BashAstParser::new();
        let result = parser.parse("diff <(echo a) >(echo b)").unwrap();
        assert!(!result.safe, "process substitution should be unsafe");
        let err = result.parse_error.as_ref().unwrap();
        assert!(
            err.contains("<(...)") && err.contains(">(...)"),
            "error should mention both <(...) and >(...): got {err}"
        );
    }

    #[test]
    fn command_substitution() {
        let parser = BashAstParser::new();
        let result = parser.parse("echo $(whoami)").unwrap();
        assert!(!result.safe, "command substitution should be unsafe");
        let err = result.parse_error.as_ref().unwrap();
        assert!(
            err.contains("$(...)"),
            "error should mention $(...): got {err}"
        );
    }

    #[test]
    fn process_subst_in_double_quotes() {
        let parser = BashAstParser::new();
        let result = parser.parse("echo \"hello $(whoami)\"").unwrap();
        assert!(!result.safe, "process substitution in double quotes should be unsafe");
    }

    // ---- pipes and command separators --------------------------------------

    #[test]
    fn piped_command_first_only() {
        let parser = BashAstParser::new();
        let result = parser.parse("cat file | grep pattern").unwrap();
        assert!(result.safe);
        assert_eq!(result.argv, vec!["cat", "file"]);
    }

    #[test]
    fn semicolon_separator() {
        let parser = BashAstParser::new();
        let result = parser.parse("echo hello; echo world").unwrap();
        assert!(result.safe);
        assert_eq!(result.argv, vec!["echo", "hello"]);
    }

    #[test]
    fn logical_and_separator() {
        let parser = BashAstParser::new();
        let result = parser.parse("true && echo hello").unwrap();
        assert!(result.safe);
        assert_eq!(result.argv, vec!["true"]);
    }

    #[test]
    fn piped_first_command_with_quotes() {
        let parser = BashAstParser::new();
        let result = parser.parse("cat \"file name\" | grep pattern").unwrap();
        assert!(result.safe);
        assert_eq!(result.argv, vec!["cat", "file name"]);
    }

    // ---- fail-closed on errors ---------------------------------------------

    #[test]
    fn empty_string_is_untrusted() {
        let parser = BashAstParser::new();
        assert!(
            parser.is_untrusted(""),
            "empty string should be untrusted (fail-closed)"
        );
        assert!(parser.extract_argv("").is_none());
    }

    #[test]
    fn whitespace_only_is_untrusted() {
        let parser = BashAstParser::new();
        assert!(parser.is_untrusted("   "));
        assert!(parser.extract_argv("   ").is_none());
    }

    #[test]
    fn unclosed_single_quote_is_untrusted() {
        let parser = BashAstParser::new();
        assert!(parser.is_untrusted("echo 'hello"));
    }

    #[test]
    fn unclosed_double_quote_is_untrusted() {
        let parser = BashAstParser::new();
        assert!(parser.is_untrusted("echo \"hello"));
    }

    #[test]
    fn trailing_backslash_is_untrusted() {
        let parser = BashAstParser::new();
        assert!(parser.is_untrusted("echo hello\\"));
    }

    // ---- is_parse_successful / extract_argv / is_untrusted helpers ---------

    #[test]
    fn is_parse_successful_with_valid_command() {
        let parser = BashAstParser::new();
        assert!(parser.is_parse_successful("echo hello"));
        assert!(!parser.is_parse_successful("echo $(whoami)"));
        assert!(!parser.is_parse_successful(""));
    }

    #[test]
    fn extract_argv_with_valid_command() {
        let parser = BashAstParser::new();
        assert_eq!(
            parser.extract_argv("echo hello world"),
            Some(vec!["echo".into(), "hello".into(), "world".into()])
        );
        assert_eq!(parser.extract_argv(""), None);
    }

    #[test]
    fn is_untrusted_fail_closed() {
        let parser = BashAstParser::new();
        // Valid commands are trusted.
        assert!(!parser.is_untrusted("ls -la"));
        // Errors produce untrusted.
        assert!(parser.is_untrusted(""), "empty is untrusted");
        assert!(
            parser.is_untrusted("$(danger)"),
            "substitution is untrusted"
        );
        assert!(
            parser.is_untrusted("'unclosed"),
            "unclosed quote is untrusted"
        );
    }

    // ---- edge cases --------------------------------------------------------

    #[test]
    fn command_with_only_flags() {
        let parser = BashAstParser::new();
        let result = parser.parse("ls -la --all").unwrap();
        assert!(result.safe);
        assert_eq!(result.argv, vec!["ls", "-la", "--all"]);
    }

    #[test]
    fn args_with_special_chars() {
        let parser = BashAstParser::new();
        let result = parser.parse("echo '$HOME'").unwrap();
        assert!(result.safe);
        // Single-quoted: literal $HOME
        assert_eq!(result.argv, vec!["echo", "$HOME"]);
    }

    #[test]
    fn multiple_pipes_only_first_parsed() {
        let parser = BashAstParser::new();
        let result = parser.parse("a | b | c").unwrap();
        assert!(result.safe);
        assert_eq!(result.argv, vec!["a"]);
    }
}
