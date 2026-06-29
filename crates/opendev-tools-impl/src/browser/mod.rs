//! Browser automation tool — headless browser interaction.
//!
//! Provides browser automation actions (navigate, click, type, fill,
//! screenshot, get_text, evaluate JS, etc.) using a pluggable backend.
//!
//! Currently uses the HTTP-only backend (`HttpBrowserBackend`) by default.
//! When a CDP/Playwright endpoint is configured, the CDP backend
//! (`CdpBrowserBackend`) will provide full browser automation.

pub(crate) mod cdp_backend;

use std::collections::HashMap;

use opendev_tools_core::{BaseTool, ToolContext, ToolDisplayMeta, ToolResult};

use cdp_backend::{select_backend, BrowserBackend};

/// Maximum page body size to process (5 MB).
const MAX_PAGE_SIZE: usize = 5 * 1024 * 1024;

/// Maximum text content length to return.
const MAX_TEXT_LENGTH: usize = 5000;

/// Default action timeout in milliseconds.
const DEFAULT_TIMEOUT_MS: u64 = 10_000;

/// Available browser actions.
const AVAILABLE_ACTIONS: &[&str] = &[
    "navigate",
    "get_text",
    "screenshot",
    "evaluate",
    "back",
    "forward",
    "reload",
    "tabs_list",
    "tab_close",
    "click",
    "type",
    "fill",
    "wait",
];

/// Tool for browser automation.
#[derive(Debug)]
pub struct BrowserTool;

#[async_trait::async_trait]
impl BaseTool for BrowserTool {
    fn name(&self) -> &str {
        "browser"
    }

    fn description(&self) -> &str {
        "Interactive browser automation. Supports actions: navigate, click, type, fill, \
         screenshot, get_text, wait, evaluate, tabs_list, tab_close, back, forward, reload."
    }

    fn parameter_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "description": "Browser action to perform",
                    "enum": AVAILABLE_ACTIONS
                },
                "target": {
                    "type": "string",
                    "description": "Target for the action (URL, CSS selector, JS expression)"
                },
                "value": {
                    "type": "string",
                    "description": "Value for the action (text to type, JS to evaluate)"
                },
                "timeout": {
                    "type": "integer",
                    "description": "Action timeout in milliseconds (default: 10000)"
                }
            },
            "required": ["action"]
        })
    }

    fn category(&self) -> opendev_tools_core::ToolCategory {
        opendev_tools_core::ToolCategory::Web
    }

    fn truncation_rule(&self) -> Option<opendev_tools_core::TruncationRule> {
        Some(opendev_tools_core::TruncationRule::head(5000))
    }

    async fn execute(
        &self,
        args: HashMap<String, serde_json::Value>,
        ctx: &ToolContext,
    ) -> ToolResult {
        let action = match args.get("action").and_then(|v| v.as_str()) {
            Some(a) => a,
            None => return ToolResult::fail("action is required"),
        };

        let target = args.get("target").and_then(|v| v.as_str());
        let value = args.get("value").and_then(|v| v.as_str());
        let _timeout = args.get("timeout").and_then(|v| v.as_u64()).unwrap_or(DEFAULT_TIMEOUT_MS);

        // Select the active backend and dispatch
        let backend: Box<dyn BrowserBackend> = select_backend();

        match action {
            "navigate" => {
                let url = match target {
                    Some(u) if !u.is_empty() => u,
                    _ => return ToolResult::fail("URL is required for navigate"),
                };
                backend.navigate(url, ctx).await
            }
            "get_text" => backend.get_text(target, ctx).await,
            "screenshot" => backend.screenshot(target, ctx).await,
            "click" => {
                let selector = match target {
                    Some(s) if !s.is_empty() => s,
                    _ => return ToolResult::fail("CSS selector is required for click"),
                };
                backend.click(selector).await
            }
            "type" => {
                let selector = match target {
                    Some(s) if !s.is_empty() => s,
                    _ => return ToolResult::fail("CSS selector is required for type"),
                };
                let text = match value {
                    Some(v) => v,
                    None => return ToolResult::fail("value (text) is required for type"),
                };
                backend.type_text(selector, text).await
            }
            "fill" => {
                let selector = match target {
                    Some(s) if !s.is_empty() => s,
                    _ => return ToolResult::fail("CSS selector is required for fill"),
                };
                let text = match value {
                    Some(v) => v,
                    None => return ToolResult::fail("value (text) is required for fill"),
                };
                backend.fill(selector, text).await
            }
            "wait" => {
                let selector = match target {
                    Some(s) if !s.is_empty() => s,
                    _ => return ToolResult::fail("CSS selector is required for wait"),
                };
                backend.wait(selector).await
            }
            "evaluate" => {
                let js_code = value.or(target);
                match js_code {
                    Some(code) if !code.is_empty() => backend.evaluate(code).await,
                    _ => ToolResult::fail("JavaScript expression is required for evaluate"),
                }
            }
            "tabs_list" => backend.tabs_list().await,
            "tab_close" => backend.tab_close(target).await,
            "back" => backend.back().await,
            "forward" => backend.forward().await,
            "reload" => backend.reload().await,
            other => ToolResult::fail(format!(
                "Unknown browser action: {other}. Available: {}",
                AVAILABLE_ACTIONS.join(", ")
            )),
        }
    }

    fn display_meta(&self) -> Option<ToolDisplayMeta> {
        Some(ToolDisplayMeta {
            verb: "Browse",
            label: "page",
            category: "Web",
            primary_arg_keys: &["action", "target"],
        })
    }
}

// ---------------------------------------------------------------------------
// HTTP client & HTML utilities (shared across backends)
// ---------------------------------------------------------------------------

/// Build an HTTP client with browser-like settings.
fn build_client() -> Result<reqwest::Client, reqwest::Error> {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .redirect(reqwest::redirect::Policy::limited(10))
        .user_agent(
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) \
             AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
        )
        .build()
}

/// Normalize a URL, adding `https://` if no scheme is present.
fn normalize_url(url: &str) -> String {
    let url = url.trim();
    if url.starts_with("https://") || url.starts_with("http://") {
        return url.to_string();
    }
    if url.starts_with("https:/") && !url.starts_with("https://") {
        return url.replacen("https:/", "https://", 1);
    }
    if url.starts_with("http:/") && !url.starts_with("http://") {
        return url.replacen("http:/", "http://", 1);
    }
    format!("https://{url}")
}

/// Extract the `<title>` from HTML.
fn extract_title(html: &str) -> Option<String> {
    let lower = html.to_lowercase();
    let start = lower.find("<title")?;
    let rest = &html[start..];
    let tag_end = rest.find('>')?;
    let after_tag = &rest[tag_end + 1..];
    let end = after_tag.find('<')?;
    let title = after_tag[..end].trim().to_string();
    if title.is_empty() { None } else { Some(html_decode(&title)) }
}

/// Extract visible text from HTML, stripping tags, scripts, and styles.
fn extract_visible_text(html: &str) -> String {
    let mut result = String::with_capacity(html.len() / 2);
    let mut in_tag = false;
    let mut in_script = false;
    let mut in_style = false;
    let lower = html.to_lowercase();
    let chars: Vec<char> = html.chars().collect();
    let lower_chars: Vec<char> = lower.chars().collect();

    let mut i = 0;
    while i < chars.len() {
        if !in_tag && chars[i] == '<' {
            in_tag = true;
            // Check if entering script or style
            let remaining: String = lower_chars[i..].iter().take(20).collect();
            if remaining.starts_with("<script") {
                in_script = true;
            } else if remaining.starts_with("<style") {
                in_style = true;
            } else if remaining.starts_with("</script") {
                in_script = false;
            } else if remaining.starts_with("</style") {
                in_style = false;
            }
        } else if in_tag && chars[i] == '>' {
            in_tag = false;
            // Add space to separate content from different tags
            if !result.ends_with(' ') && !result.ends_with('\n') {
                result.push(' ');
            }
        } else if !in_tag && !in_script && !in_style {
            result.push(chars[i]);
        }
        i += 1;
    }

    // Decode HTML entities and collapse whitespace
    let decoded = html_decode(&result);
    let lines: Vec<&str> = decoded.lines().map(|l| l.trim()).filter(|l| !l.is_empty()).collect();
    lines.join("\n")
}

/// Decode common HTML entities.
fn html_decode(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
        .replace("&nbsp;", " ")
}

/// Select the default browser backend.
///
/// Currently returns the HTTP-only backend. When a CDP endpoint is
/// configured in settings, this will detect and return the appropriate backend.
fn select_default_backend() -> Box<dyn BrowserBackend> {
    select_backend()
}

#[cfg(test)]
mod tests;
