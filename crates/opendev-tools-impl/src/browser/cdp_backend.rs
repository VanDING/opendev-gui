//! Browser backend abstraction — separates HTTP-only mode from CDP/Playwright.
//!
//! Defines the [`BrowserBackend`] trait that the browser tool delegates to,
//! and provides two implementations:
//!
//! - [`HttpBrowserBackend`] — the existing reqwest-based HTTP fetcher
//!   (no JavaScript, no real browser session).
//! - [`CdpBrowserBackend`] — a placeholder that guides users to install
//!   Playwright for full browser automation.

use std::collections::HashMap;

use opendev_tools_core::{ToolContext, ToolResult};

use super::{build_client, extract_title, extract_visible_text, normalize_url, MAX_PAGE_SIZE, MAX_TEXT_LENGTH};

/// Browser backend trait — each method maps to a browser tool action.
#[async_trait::async_trait]
pub trait BrowserBackend: Send + Sync + std::fmt::Debug {
    /// Navigate to a URL and return page info.
    async fn navigate(&self, url: &str, ctx: &ToolContext) -> ToolResult;

    /// Get text content from the page or a specific element.
    async fn get_text(&self, target: Option<&str>, ctx: &ToolContext) -> ToolResult;

    /// Capture a screenshot (or HTML snapshot).
    async fn screenshot(&self, target: Option<&str>, ctx: &ToolContext) -> ToolResult;

    /// Click an element (CSS selector).
    async fn click(&self, selector: &str) -> ToolResult;

    /// Type text into an element.
    async fn type_text(&self, selector: &str, value: &str) -> ToolResult;

    /// Fill a form field.
    async fn fill(&self, selector: &str, value: &str) -> ToolResult;

    /// Wait for an element.
    async fn wait(&self, selector: &str) -> ToolResult;

    /// Evaluate JavaScript.
    async fn evaluate(&self, js_code: &str) -> ToolResult;

    /// List open tabs.
    async fn tabs_list(&self) -> ToolResult;

    /// Close a tab.
    async fn tab_close(&self, target: Option<&str>) -> ToolResult;

    /// Navigate back.
    async fn back(&self) -> ToolResult;

    /// Navigate forward.
    async fn forward(&self) -> ToolResult;

    /// Reload the current page.
    async fn reload(&self) -> ToolResult;
}

// ---------------------------------------------------------------------------
// HTTP Browser Backend (current reqwest-based implementation)
// ---------------------------------------------------------------------------

/// HTTP-only browser backend using `reqwest`.
///
/// Fetches pages via HTTP GET — no JavaScript, no persistent session.
/// Suitable for simple content retrieval.
#[derive(Debug)]
pub struct HttpBrowserBackend;

#[async_trait::async_trait]
impl BrowserBackend for HttpBrowserBackend {
    async fn navigate(&self, url: &str, _ctx: &ToolContext) -> ToolResult {
        let url = normalize_url(url);
        let client = match build_client() {
            Ok(c) => c,
            Err(e) => return ToolResult::fail(format!("Failed to create HTTP client: {e}")),
        };

        let response = match client.get(&url).send().await {
            Ok(r) => r,
            Err(e) => return ToolResult::fail(format!("Navigation failed: {e}")),
        };

        let status = response.status().as_u16();
        let final_url = response.url().to_string();

        let body = match response.text().await {
            Ok(t) => t,
            Err(e) => return ToolResult::fail(format!("Failed to read page: {e}")),
        };

        let title = extract_title(&body).unwrap_or_else(|| "Untitled".to_string());

        let mut metadata = HashMap::new();
        metadata.insert("status".into(), serde_json::json!(status));
        metadata.insert("url".into(), serde_json::json!(final_url));
        metadata.insert("title".into(), serde_json::json!(title));

        ToolResult::ok_with_metadata(
            format!("Navigated to: {url}\nTitle: {title}\nURL: {final_url}"),
            metadata,
        )
    }

    async fn get_text(&self, target: Option<&str>, _ctx: &ToolContext) -> ToolResult {
        let url = match target {
            Some(t) if t.starts_with("http://") || t.starts_with("https://") => t,
            Some(selector) => {
                return ToolResult::fail(format!(
                    "CSS selector '{selector}' requires an active browser session. \
                     Use 'navigate' action first, then provide a URL for get_text, or \
                     use the web_fetch tool instead."
                ));
            }
            None => {
                return ToolResult::fail("Target (URL or CSS selector) is required for get_text");
            }
        };

        let client = match build_client() {
            Ok(c) => c,
            Err(e) => return ToolResult::fail(format!("Failed to create HTTP client: {e}")),
        };

        let response = match client.get(url).send().await {
            Ok(r) => r,
            Err(e) => return ToolResult::fail(format!("Request failed: {e}")),
        };

        let body = match response.text().await {
            Ok(t) => t,
            Err(e) => return ToolResult::fail(format!("Failed to read response: {e}")),
        };

        let text = extract_visible_text(&body);
        let truncated = text.len() > MAX_TEXT_LENGTH;
        let text = if truncated {
            format!("{}...\n[truncated]", &text[..MAX_TEXT_LENGTH])
        } else {
            text
        };

        ToolResult::ok(text)
    }

    async fn screenshot(&self, target: Option<&str>, _ctx: &ToolContext) -> ToolResult {
        let url = match target {
            Some(u) if u.starts_with("http://") || u.starts_with("https://") => u,
            Some(_) | None => {
                return ToolResult::fail(
                    "URL is required for screenshot. Use web_screenshot tool for full \
                     browser screenshots with JavaScript rendering.",
                );
            }
        };

        let client = match build_client() {
            Ok(c) => c,
            Err(e) => return ToolResult::fail(format!("Failed to create HTTP client: {e}")),
        };

        let response = match client.get(url).send().await {
            Ok(r) => r,
            Err(e) => return ToolResult::fail(format!("Request failed: {e}")),
        };

        let body = match response.text().await {
            Ok(t) => {
                if t.len() > MAX_PAGE_SIZE {
                    t[..MAX_PAGE_SIZE].to_string()
                } else {
                    t
                }
            }
            Err(e) => return ToolResult::fail(format!("Failed to read page: {e}")),
        };

        let screenshot_dir = std::env::temp_dir().join("opendev-screenshots");
        std::fs::create_dir_all(&screenshot_dir).ok();
        let filename = format!("browser_{}.html", uuid::Uuid::new_v4());
        let path = screenshot_dir.join(&filename);

        let mut opts = std::fs::OpenOptions::new();
        opts.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            opts.mode(0o600);
        }

        let write_result = opts.open(&path).and_then(|mut f| {
            use std::io::Write;
            f.write_all(body.as_bytes())
        });

        match write_result {
            Ok(_) => {
                let mut metadata = HashMap::new();
                metadata.insert("screenshot_path".into(), serde_json::json!(path.to_string_lossy()));
                metadata.insert("format".into(), serde_json::json!("html"));
                metadata.insert(
                    "note".into(),
                    serde_json::json!(
                        "HTML snapshot saved. For rendered screenshots, use the web_screenshot tool."
                    ),
                );
                ToolResult::ok_with_metadata(
                    format!(
                        "HTML snapshot saved: {}\nPage: {url}\n\
                         Note: For visual screenshots, use the web_screenshot tool.",
                        path.display()
                    ),
                    metadata,
                )
            }
            Err(e) => ToolResult::fail(format!("Failed to save snapshot: {e}")),
        }
    }

    async fn click(&self, selector: &str) -> ToolResult {
        ToolResult::fail(format!(
            "Click on '{selector}' requires a browser session with JavaScript support. \
             Consider using the web_fetch tool for content retrieval, or the bash tool \
             to run a headless browser script."
        ))
    }

    async fn type_text(&self, selector: &str, _value: &str) -> ToolResult {
        ToolResult::fail(format!(
            "Typing into '{selector}' requires a browser session with JavaScript support. \
             Consider using curl/wget via the bash tool for form submission."
        ))
    }

    async fn fill(&self, selector: &str, _value: &str) -> ToolResult {
        ToolResult::fail(format!(
            "Filling '{selector}' requires a browser session with JavaScript support."
        ))
    }

    async fn wait(&self, selector: &str) -> ToolResult {
        ToolResult::fail(format!(
            "Waiting for '{selector}' requires a browser session with JavaScript support."
        ))
    }

    async fn evaluate(&self, _js_code: &str) -> ToolResult {
        ToolResult::fail(
            "JavaScript evaluation requires a browser session. \
             Consider using the bash tool to run Node.js scripts."
                .to_string(),
        )
    }

    async fn tabs_list(&self) -> ToolResult {
        ToolResult::ok(
            "No browser context open (HTTP-only mode). Use 'navigate' to fetch a page.",
        )
    }

    async fn tab_close(&self, _target: Option<&str>) -> ToolResult {
        ToolResult::ok("No browser context open (HTTP-only mode).")
    }

    async fn back(&self) -> ToolResult {
        ToolResult::fail("Browser history navigation requires a persistent browser session.")
    }

    async fn forward(&self) -> ToolResult {
        ToolResult::fail("Browser history navigation requires a persistent browser session.")
    }

    async fn reload(&self) -> ToolResult {
        ToolResult::fail(
            "Reload requires a persistent browser session. Use 'navigate' to re-fetch a URL.",
        )
    }
}

// ---------------------------------------------------------------------------
// CDP / Playwright Browser Backend (placeholder)
// ---------------------------------------------------------------------------

/// CDP-based browser backend placeholder.
///
/// Shows a hint to install Playwright for full browser automation.
/// When implemented, this will communicate with a Playwright/CDP bridge process.
#[derive(Debug)]
pub struct CdpBrowserBackend;

#[async_trait::async_trait]
impl BrowserBackend for CdpBrowserBackend {
    async fn navigate(&self, _url: &str, _ctx: &ToolContext) -> ToolResult {
        cdp_hint()
    }

    async fn get_text(&self, _target: Option<&str>, _ctx: &ToolContext) -> ToolResult {
        cdp_hint()
    }

    async fn screenshot(&self, _target: Option<&str>, _ctx: &ToolContext) -> ToolResult {
        cdp_hint()
    }

    async fn click(&self, _selector: &str) -> ToolResult {
        cdp_hint()
    }

    async fn type_text(&self, _selector: &str, _value: &str) -> ToolResult {
        cdp_hint()
    }

    async fn fill(&self, _selector: &str, _value: &str) -> ToolResult {
        cdp_hint()
    }

    async fn wait(&self, _selector: &str) -> ToolResult {
        cdp_hint()
    }

    async fn evaluate(&self, _js_code: &str) -> ToolResult {
        cdp_hint()
    }

    async fn tabs_list(&self) -> ToolResult {
        cdp_hint()
    }

    async fn tab_close(&self, _target: Option<&str>) -> ToolResult {
        cdp_hint()
    }

    async fn back(&self) -> ToolResult {
        cdp_hint()
    }

    async fn forward(&self) -> ToolResult {
        cdp_hint()
    }

    async fn reload(&self) -> ToolResult {
        cdp_hint()
    }
}

fn cdp_hint() -> ToolResult {
    ToolResult::fail(
        "Full browser automation requires a CDP/Playwright bridge. \
         Install playwright: 'npx playwright install chromium' and configure \
         the CDP endpoint in OpenDev settings.",
    )
}

/// Select the active backend based on configuration.
///
/// Currently always returns `HttpBrowserBackend`. When a CDP endpoint is
/// configured, this will return `CdpBrowserBackend`.
pub fn select_backend() -> Box<dyn BrowserBackend> {
    // TODO: Check config for CDP endpoint URL. If set, return CdpBrowserBackend.
    Box::new(HttpBrowserBackend)
}
