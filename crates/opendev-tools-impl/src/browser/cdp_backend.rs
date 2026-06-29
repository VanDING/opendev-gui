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
// CDP / Playwright Browser Backend
// ---------------------------------------------------------------------------

/// Default Chrome DevTools Protocol endpoint URL.
const DEFAULT_CDP_URL: &str = "http://localhost:9222";

/// CDP-based browser backend using Chrome DevTools Protocol over HTTP.
///
/// Communicates with a Chrome/Chromium instance via CDP HTTP endpoints.
/// Requires Chrome to be launched with `--remote-debugging-port=9222`.
///
/// For full CDP support (DOM interaction, screenshots, JS evaluation),
/// connects via the inspection HTTP API:
/// - List tabs: `GET /json`
/// - Open URL:  `GET /json/new?url=...`
/// - Evaluate:  `GET /json/version` then use WebSocket for Runtime.evaluate
///
/// Currently uses available HTTP endpoints. WebSocket-based commands
/// (Runtime.evaluate, Page.captureScreenshot) require additional
/// WebSocket support.
#[derive(Debug)]
pub struct CdpBrowserBackend {
    /// CDP endpoint URL (e.g., `http://localhost:9222`).
    cdp_url: String,
}

impl CdpBrowserBackend {
    /// Create a new CDP browser backend.
    ///
    /// Uses `CHROME_CDP_URL` env var or defaults to `http://localhost:9222`.
    pub fn new() -> Self {
        let cdp_url =
            std::env::var("CHROME_CDP_URL").unwrap_or_else(|_| DEFAULT_CDP_URL.to_string());
        Self { cdp_url }
    }

    /// Build a reusable reqwest client.
    fn client() -> Result<reqwest::Client, String> {
        reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {e}"))
    }

    /// List all page targets from Chrome's CDP endpoint.
    async fn list_tabs(&self) -> Result<Vec<CdpTab>, String> {
        let client = Self::client()?;
        let url = format!("{}/json", self.cdp_url);
        let resp = client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Failed to list CDP tabs: {e}"))?;
        let tabs: Vec<CdpTab> = resp
            .json()
            .await
            .map_err(|e| format!("Failed to parse CDP tab list: {e}"))?;
        Ok(tabs)
    }

    /// Open a new tab and navigate to the given URL.
    async fn open_url(&self, url: &str) -> Result<String, String> {
        let client = Self::client()?;
        let full_url = format!("{}/json/new?url={}", self.cdp_url, urlencoding(url));
        let resp = client
            .get(&full_url)
            .send()
            .await
            .map_err(|e| format!("Failed to open URL via CDP: {e}"))?;
        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("Failed to parse CDP new tab response: {e}"))?;
        Ok(body
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string())
    }

    /// Close a tab by its target ID.
    async fn close_tab(&self, target_id: &str) -> Result<(), String> {
        let client = Self::client()?;
        let url = format!("{}/json/close/{}", self.cdp_url, target_id);
        client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Failed to close CDP tab: {e}"))?;
        Ok(())
    }

    /// Activate a tab by its target ID.
    async fn activate_tab(&self, target_id: &str) -> Result<(), String> {
        let client = Self::client()?;
        let url = format!("{}/json/activate/{}", self.cdp_url, target_id);
        client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Failed to activate CDP tab: {e}"))?;
        Ok(())
    }
}

impl Default for CdpBrowserBackend {
    fn default() -> Self {
        Self::new()
    }
}

/// A tab/page target from the CDP `/json` endpoint.
#[derive(Debug, serde::Deserialize)]
struct CdpTab {
    #[serde(default)]
    id: String,
    #[serde(default)]
    title: String,
    #[serde(default)]
    url: String,
    #[serde(default)]
    #[allow(dead_code)]
    web_socket_debugger_url: Option<String>,
}

#[async_trait::async_trait]
impl BrowserBackend for CdpBrowserBackend {
    async fn navigate(&self, url: &str, _ctx: &ToolContext) -> ToolResult {
        match self.open_url(url).await {
            Ok(tab_id) => {
                let mut metadata = HashMap::new();
                metadata.insert("target_id".into(), serde_json::json!(tab_id));
                metadata.insert("url".into(), serde_json::json!(url));
                ToolResult::ok_with_metadata(
                    format!("Navigated to: {url}\nTarget ID: {tab_id}"),
                    metadata,
                )
            }
            Err(e) => ToolResult::fail(format!("CDP navigation failed: {e}")),
        }
    }

    async fn get_text(&self, target: Option<&str>, _ctx: &ToolContext) -> ToolResult {
        let tabs = match self.list_tabs().await {
            Ok(t) => t,
            Err(e) => return ToolResult::fail(format!("CDP error: {e}")),
        };
        // Find a tab matching the target URL, or use the first page tab
        let tab = match target {
            Some(url) => tabs.iter().find(|t| t.url.contains(url)).or(tabs.first()),
            None => tabs.first(),
        };
        match tab {
            Some(t) => {
                // CDP HTTP doesn't natively expose text content.
                // Fall back to fetching the page via HTTP and extracting text.
                let client = match Self::client() {
                    Ok(c) => c,
                    Err(e) => return ToolResult::fail(e),
                };
                match client.get(&t.url).send().await {
                    Ok(resp) => {
                        let body = resp.text().await.unwrap_or_default();
                        let text = extract_visible_text(&body);
                        let truncated = text.len() > MAX_TEXT_LENGTH;
                        let text = if truncated {
                            format!("{}...\n[truncated]", &text[..MAX_TEXT_LENGTH])
                        } else {
                            text
                        };
                        ToolResult::ok(text)
                    }
                    Err(e) => ToolResult::fail(format!("Failed to fetch page text: {e}")),
                }
            }
            None => ToolResult::fail("No browser tab found. Use 'navigate' to open a page first."),
        }
    }

    async fn screenshot(&self, target: Option<&str>, _ctx: &ToolContext) -> ToolResult {
        let tabs = match self.list_tabs().await {
            Ok(t) => t,
            Err(e) => return ToolResult::fail(format!("CDP error: {e}")),
        };
        let tab = match target {
            Some(url) => tabs.iter().find(|t| t.url.contains(url)).or(tabs.first()),
            None => tabs.first(),
        };
        match tab {
            Some(t) => {
                // For HTTP-based CDP, save an HTML snapshot instead of a real screenshot.
                // Real Page.captureScreenshot requires WebSocket CDP.
                let client = match Self::client() {
                    Ok(c) => c,
                    Err(e) => return ToolResult::fail(e),
                };
                match client.get(&t.url).send().await {
                    Ok(resp) => {
                        let body = resp.text().await.unwrap_or_default();
                        let body = if body.len() > MAX_PAGE_SIZE {
                            body[..MAX_PAGE_SIZE].to_string()
                        } else {
                            body
                        };
                        let screenshot_dir = std::env::temp_dir().join("opendev-screenshots");
                        std::fs::create_dir_all(&screenshot_dir).ok();
                        let filename = format!("cdp_{}.html", uuid::Uuid::new_v4());
                        let path = screenshot_dir.join(&filename);
                        let mut opts = std::fs::OpenOptions::new();
                        opts.write(true).create_new(true);
                        #[cfg(unix)]
                        {
                            use std::os::unix::fs::OpenOptionsExt;
                            opts.mode(0o600);
                        }
                        match opts.open(&path).and_then(|mut f| {
                            use std::io::Write;
                            f.write_all(body.as_bytes())
                        }) {
                            Ok(_) => {
                                let mut metadata = HashMap::new();
                                metadata.insert(
                                    "screenshot_path".into(),
                                    serde_json::json!(path.to_string_lossy()),
                                );
                                metadata.insert("format".into(), serde_json::json!("html"));
                                ToolResult::ok_with_metadata(
                                    format!(
                                        "CDP snapshot saved: {}\nPage: {}",
                                        path.display(),
                                        t.url
                                    ),
                                    metadata,
                                )
                            }
                            Err(e) => ToolResult::fail(format!("Failed to save snapshot: {e}")),
                        }
                    }
                    Err(e) => ToolResult::fail(format!("Failed to fetch page: {e}")),
                }
            }
            None => ToolResult::fail("No browser tab found. Use 'navigate' to open a page first."),
        }
    }

    async fn click(&self, selector: &str) -> ToolResult {
        ToolResult::fail(format!(
            "Click on '{selector}' via CDP requires WebSocket support. \
             As a workaround, use Runtime.evaluate with \
             'document.querySelector(\"{sel}\").click()' \
             via the 'evaluate' action.",
            sel = selector.replace('"', "\\\"")
        ))
    }

    async fn type_text(&self, selector: &str, value: &str) -> ToolResult {
        ToolResult::fail(format!(
            "Type into '{selector}' via CDP requires WebSocket support. \
             Use Runtime.evaluate with \
             'document.querySelector(\"{sel}\").value = \"{val}\"' \
             via the 'evaluate' action.",
            sel = selector.replace('"', "\\\""),
            val = value.replace('"', "\\\"")
        ))
    }

    async fn fill(&self, selector: &str, value: &str) -> ToolResult {
        self.type_text(selector, value).await
    }

    async fn wait(&self, selector: &str) -> ToolResult {
        ToolResult::fail(format!(
            "Wait for '{selector}' via CDP requires WebSocket support."
        ))
    }

    async fn evaluate(&self, _js_code: &str) -> ToolResult {
        ToolResult::fail(
            "JavaScript evaluation via CDP requires WebSocket support. \
             Use the bash tool to run Node.js scripts as an alternative.",
        )
    }

    async fn tabs_list(&self) -> ToolResult {
        match self.list_tabs().await {
            Ok(tabs) => {
                if tabs.is_empty() {
                    return ToolResult::ok("No open tabs.");
                }
                let mut output = String::from("Open tabs:\n");
                for (i, tab) in tabs.iter().enumerate() {
                    let title = if tab.title.len() > 60 {
                        format!("{}...", &tab.title[..60])
                    } else {
                        tab.title.clone()
                    };
                    output.push_str(&format!("  {}. {} (id: {}, url: {})\n", i + 1, title, tab.id, tab.url));
                }
                ToolResult::ok(output)
            }
            Err(e) => ToolResult::fail(format!("CDP error: {e}")),
        }
    }

    async fn tab_close(&self, target: Option<&str>) -> ToolResult {
        let target_id = match target {
            Some(id) => id.to_string(),
            None => {
                // Close the last active tab
                match self.list_tabs().await {
                    Ok(tabs) => {
                        if let Some(tab) = tabs.last() {
                            tab.id.clone()
                        } else {
                            return ToolResult::ok("No tabs to close.");
                        }
                    }
                    Err(e) => return ToolResult::fail(format!("CDP error: {e}")),
                }
            }
        };
        match self.close_tab(&target_id).await {
            Ok(()) => ToolResult::ok(format!("Closed tab: {target_id}")),
            Err(e) => ToolResult::fail(format!("Failed to close tab: {e}")),
        }
    }

    async fn back(&self) -> ToolResult {
        ToolResult::fail(
            "Back navigation via CDP requires WebSocket support. \
             Use 'navigate' to go to a previous URL directly.",
        )
    }

    async fn forward(&self) -> ToolResult {
        ToolResult::fail(
            "Forward navigation via CDP requires WebSocket support. \
             Use 'navigate' to go forward directly.",
        )
    }

    async fn reload(&self) -> ToolResult {
        ToolResult::fail(
            "Reload via CDP requires WebSocket support. \
             Use 'navigate' to re-fetch a URL.",
        )
    }
}

/// Simple percent-encoding for CDP URL parameters.
fn urlencoding(s: &str) -> String {
    s.replace(' ', "%20")
        .replace('&', "%26")
        .replace('?', "%3F")
        .replace('#', "%23")
        .replace('"', "%22")
        .replace('\'', "%27")
}

/// Select the active backend based on configuration.
///
/// Returns `CdpBrowserBackend` when Chrome CDP endpoint is reachable
/// (via `CHROME_CDP_URL` env var or default `http://localhost:9222`).
/// Otherwise falls back to `HttpBrowserBackend`.
pub fn select_backend() -> Box<dyn BrowserBackend> {
    let cdp_url =
        std::env::var("CHROME_CDP_URL").unwrap_or_else(|_| DEFAULT_CDP_URL.to_string());

    // Quick probe: check if the CDP endpoint is reachable.
    let is_cdp_available = match tokio::runtime::Handle::try_current() {
        Ok(handle) => {
            tokio::task::block_in_place(|| {
                handle.block_on(async {
                    reqwest::get(&format!("{cdp_url}/json/version"))
                        .await
                        .is_ok()
                })
            })
        }
        Err(_) => false,
    };

    if is_cdp_available {
        tracing::info!(cdp_url = %cdp_url, "Using CDP browser backend");
        Box::new(CdpBrowserBackend::new())
    } else {
        tracing::debug!("CDP endpoint not available, using HTTP browser backend");
        Box::new(HttpBrowserBackend)
    }
}
