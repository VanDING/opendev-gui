//! Web search tool — search the web via Brave Search API, with fallback to
//! DuckDuckGo HTML scraping when no API key is configured.
//!
//! Backend priority:
//! 1. Brave Search API (requires BRAVE_SEARCH_API_KEY env var)
//! 2. DuckDuckGo HTML scraping (no API key required, degraded mode)

mod parser;

use std::collections::HashMap;

use opendev_tools_core::{BaseTool, ToolContext, ToolResult};

use parser::{filter_by_domain, parse_ddg_html, urlencoded as urlencode, SearchResult};

/// Default number of search results to return.
const DEFAULT_MAX_RESULTS: usize = 10;

/// Maximum body size to read from DuckDuckGo (256 KB).
const MAX_BODY_SIZE: usize = 256 * 1024;

/// Brave Search API endpoint.
const BRAVE_API_URL: &str = "https://api.search.brave.com/res/v1/web/search";

/// Environment variable for Brave Search API key.
const BRAVE_API_KEY_ENV: &str = "BRAVE_SEARCH_API_KEY";

/// Environment variable for SerpAPI API key.
const SERPAPI_API_KEY_ENV: &str = "SERPAPI_API_KEY";

/// SerpAPI search endpoint.
const SERPAPI_API_URL: &str = "https://serpapi.com/search";

/// Search backend options.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchBackend {
    /// Brave Search API (primary).
    Brave,
    /// SerpAPI (future, not yet implemented).
    SerpApi,
    /// DuckDuckGo HTML scraping fallback.
    DdgFallback,
}

impl SearchBackend {
    fn try_available() -> Self {
        if std::env::var(BRAVE_API_KEY_ENV).ok().filter(|k| !k.is_empty()).is_some() {
            Self::Brave
        } else if std::env::var(SERPAPI_API_KEY_ENV).ok().filter(|k| !k.is_empty()).is_some() {
            Self::SerpApi
        } else {
            Self::DdgFallback
        }
    }
}

/// Tool for searching the web using Brave Search API or DuckDuckGo fallback.
#[derive(Debug)]
pub struct WebSearchTool;

#[async_trait::async_trait]
impl BaseTool for WebSearchTool {
    fn name(&self) -> &str {
        "WebSearch"
    }

    fn description(&self) -> &str {
        "Search the web. Uses Brave Search API when BRAVE_SEARCH_API_KEY is set, \
         otherwise falls back to DuckDuckGo."
    }

    fn parameter_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "Search query string"
                },
                "max_results": {
                    "type": "integer",
                    "description": "Maximum number of results (default: 10)"
                },
                "allowed_domains": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Only include results from these domains"
                },
                "blocked_domains": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Exclude results from these domains"
                }
            },
            "required": ["query"]
        })
    }

    fn is_read_only(&self, _args: &HashMap<String, serde_json::Value>) -> bool {
        true
    }

    fn is_concurrent_safe(&self, _args: &HashMap<String, serde_json::Value>) -> bool {
        true
    }

    fn category(&self) -> opendev_tools_core::ToolCategory {
        opendev_tools_core::ToolCategory::Web
    }

    fn truncation_rule(&self) -> Option<opendev_tools_core::TruncationRule> {
        Some(opendev_tools_core::TruncationRule::head(10000))
    }

    fn search_hint(&self) -> Option<&str> {
        Some("search the web for information")
    }

    async fn execute(
        &self,
        args: HashMap<String, serde_json::Value>,
        _ctx: &ToolContext,
    ) -> ToolResult {
        let query = match args.get("query").and_then(|v| v.as_str()) {
            Some(q) if !q.trim().is_empty() => q.trim(),
            _ => return ToolResult::fail("Search query is required"),
        };

        let max_results = args
            .get("max_results")
            .and_then(|v| v.as_u64())
            .map(|n| n as usize)
            .unwrap_or(DEFAULT_MAX_RESULTS);

        let allowed_domains: Vec<String> = args
            .get("allowed_domains")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_lowercase())).collect())
            .unwrap_or_default();

        let blocked_domains: Vec<String> = args
            .get("blocked_domains")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_lowercase())).collect())
            .unwrap_or_default();

        // Select backend
        let backend = SearchBackend::try_available();

        let results = match backend {
            SearchBackend::Brave => {
                match search_brave(query, max_results).await {
                    Ok(r) => r,
                    Err(e) => {
                        tracing::warn!("Brave Search API failed: {e}; falling back to DDG");
                        search_ddg(query, max_results).await
                    }
                }
            }
            SearchBackend::SerpApi => {
                match search_serpapi(query, max_results).await {
                    Ok(r) => r,
                    Err(e) => {
                        tracing::warn!("SerpAPI search failed: {e}; falling back to DDG");
                        search_ddg(query, max_results).await
                    }
                }
            }
            SearchBackend::DdgFallback => {
                search_ddg(query, max_results).await
            }
        };

        // Filter by domain
        let results = if !allowed_domains.is_empty() || !blocked_domains.is_empty() {
            filter_by_domain(results, &allowed_domains, &blocked_domains)
        } else {
            results
        };

        let result_count = results.len();

        // Format output
        let mut output_parts = Vec::new();

        let backend_label = match backend {
            SearchBackend::Brave => "Brave Search API",
            SearchBackend::SerpApi => "SerpAPI",
            SearchBackend::DdgFallback => "DuckDuckGo",
        };
        output_parts.push(format!(
            "Search results for \"{query}\" ({result_count} results, via {backend_label}):\n"
        ));

        for (i, result) in results.iter().enumerate() {
            output_parts.push(format!(
                "{}. {}\n   {}\n   {}\n",
                i + 1,
                result.title,
                result.url,
                result.snippet
            ));
        }

        if results.is_empty() {
            output_parts.push("No results found.".to_string());
        }

        let output = output_parts.join("");

        let mut metadata = HashMap::new();
        metadata.insert("query".into(), serde_json::json!(query));
        metadata.insert("result_count".into(), serde_json::json!(result_count));
        metadata.insert("results".into(), serde_json::to_value(&results).unwrap_or_default());
        metadata.insert("backend".into(), serde_json::json!(format!("{backend:?}")));

        ToolResult::ok_with_metadata(output, metadata)
    }
}

// ---------------------------------------------------------------------------
// Backend implementations
// ---------------------------------------------------------------------------

/// Search via Brave Search API.
async fn search_brave(query: &str, max_results: usize) -> Result<Vec<SearchResult>, String> {
    let api_key = std::env::var(BRAVE_API_KEY_ENV)
        .map_err(|_| "BRAVE_SEARCH_API_KEY not set".to_string())?;

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {e}"))?;

    let url = format!("{}?q={}&count={}", BRAVE_API_URL, urlencode(query), max_results);
    let response = client
        .get(&url)
        .header("X-Subscription-Token", &api_key)
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|e| format!("Brave Search request failed: {e}"))?;

    if !response.status().is_success() {
        let status = response.status();
        return Err(format!("Brave Search returned HTTP {status}"));
    }

    let body: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse Brave Search response: {e}"))?;

    let results = body
        .get("web")
        .and_then(|w| w.get("results"))
        .and_then(|r| r.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|item| {
                    let title = item.get("title")?.as_str()?.to_string();
                    let url = item.get("url")?.as_str()?.to_string();
                    let snippet = item
                        .get("description")
                        .and_then(|d| d.as_str())
                        .unwrap_or("")
                        .to_string();
                    Some(SearchResult { title, url, snippet })
                })
                .take(max_results)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    Ok(results)
}

/// Search via SerpAPI. Requires `SERPAPI_API_KEY` env var.
async fn search_serpapi(query: &str, max_results: usize) -> Result<Vec<SearchResult>, String> {
    let api_key = std::env::var(SERPAPI_API_KEY_ENV)
        .map_err(|_| "SERPAPI_API_KEY not set".to_string())?;

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {e}"))?;

    let url = format!(
        "{}?q={}&api_key={}&engine=google&num={}",
        SERPAPI_API_URL,
        urlencode(query),
        api_key,
        max_results
    );
    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("SerpAPI request failed: {e}"))?;

    if !response.status().is_success() {
        let status = response.status();
        return Err(format!("SerpAPI returned HTTP {status}"));
    }

    let body: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse SerpAPI response: {e}"))?;

    let results = body
        .get("organic_results")
        .and_then(|r| r.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|item| {
                    let title = item.get("title")?.as_str()?.to_string();
                    let url = item.get("link")?.as_str()?.to_string();
                    let snippet = item
                        .get("snippet")
                        .and_then(|d| d.as_str())
                        .unwrap_or("")
                        .to_string();
                    Some(SearchResult { title, url, snippet })
                })
                .take(max_results)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    Ok(results)
}

/// Search via DuckDuckGo HTML scraping (fallback).
async fn search_ddg(query: &str, max_results: usize) -> Vec<SearchResult> {
    let encoded_query = urlencode(query);
    let url = format!("https://html.duckduckgo.com/html/?q={encoded_query}");

    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .redirect(reqwest::redirect::Policy::limited(5))
        .user_agent(
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) \
             AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
        )
        .build()
    {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let response = match client.get(&url).send().await {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };

    if !response.status().is_success() {
        return Vec::new();
    }

    let body = match response.text().await {
        Ok(t) => {
            if t.len() > MAX_BODY_SIZE {
                t[..MAX_BODY_SIZE].to_string()
            } else {
                t
            }
        }
        Err(_) => return Vec::new(),
    };

    let mut results = parse_ddg_html(&body);
    results.truncate(max_results);
    results
}

#[cfg(test)]
mod tests;
