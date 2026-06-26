//! HTML-to-markdown conversion utilities.
//!
//! Regex-based extraction rather than a full DOM parser. Handles the most
//! common HTML patterns: headings, paragraphs, links, lists, code blocks,
//! emphasis, and removes scripts/styles/navigation.
//!
//! All regexes are compiled once via `LazyLock` — the original code
//! recompiled ~17 patterns per invocation.

use std::sync::LazyLock;

use regex::Regex;

// ── Static regexes (compiled once) ─────────────────────────────

static TAG_REMOVERS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    ["script", "style", "nav", "footer", "header", "noscript", "svg"]
        .iter()
        .filter_map(|tag| Regex::new(&format!(r"(?is)<{tag}[^>]*>.*?</{tag}>")).ok())
        .collect()
});

static COMMENT_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?s)<!--.*?-->").expect("valid regex"));

static HEADING_RES: LazyLock<Vec<(usize, Regex)>> = LazyLock::new(|| {
    (1..=6)
        .map(|level| {
            (
                level,
                Regex::new(&format!(r"(?i)<h{level}[^>]*>(.*?)</h{level}>"))
                    .expect("valid regex"),
            )
        })
        .collect()
});

static PRE_CODE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?is)<pre[^>]*>\s*<code[^>]*>(.*?)</code>\s*</pre>").expect("valid regex"));

static PRE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?is)<pre[^>]*>(.*?)</pre>").expect("valid regex"));

static CODE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)<code[^>]*>(.*?)</code>").expect("valid regex"));

static LINK_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"(?i)<a[^>]*href="([^"]*)"[^>]*>(.*?)</a>"#).expect("valid regex"));

static IMG_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"(?i)<img[^>]*alt="([^"]*)"[^>]*src="([^"]*)"[^>]*/?>"#).expect("valid regex"));

static STRONG_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)<(?:strong|b)>(.*?)</(?:strong|b)>").expect("valid regex"));

static EM_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)<(?:em|i)>(.*?)</(?:em|i)>").expect("valid regex"));

static LI_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)<li[^>]*>(.*?)</li>").expect("valid regex"));

static BLOCKQUOTE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?is)<blockquote[^>]*>(.*?)</blockquote>").expect("valid regex"));

static BR_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)<br\s*/?>").expect("valid regex"));

static HR_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)<hr\s*/?>").expect("valid regex"));

static P_DIV_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)</?(?:p|div|section|article|main)[^>]*>").expect("valid regex"));

static MULTI_NEWLINE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\n{3,}").expect("valid regex"));

static MULTI_SPACE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"[ \t]{2,}").expect("valid regex"));

static TAG_STRIP_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"<[^>]*>").expect("valid regex"));

// ── Conversion ─────────────────────────────────────────────────

/// Convert HTML content to clean markdown for LLM-friendly output.
pub(super) fn html_to_markdown(html: &str) -> String {
    let mut text = html.to_string();

    // Remove script, style, nav, footer, header, noscript, svg
    for re in TAG_REMOVERS.iter() {
        text = re.replace_all(&text, "").to_string();
    }

    // Remove HTML comments
    text = COMMENT_RE.replace_all(&text, "").to_string();

    // Convert headings
    for (level, re) in HEADING_RES.iter() {
        let prefix = "#".repeat(*level);
        text = re
            .replace_all(&text, |caps: &regex::Captures| {
                format!("\n\n{prefix} {}\n\n", strip_tags(&caps[1]))
            })
            .to_string();
    }

    // Convert pre/code blocks
    text = PRE_CODE_RE
        .replace_all(&text, |caps: &regex::Captures| {
            format!("\n\n```\n{}\n```\n\n", decode_entities(&caps[1]))
        })
        .to_string();
    text = PRE_RE
        .replace_all(&text, |caps: &regex::Captures| {
            format!("\n\n```\n{}\n```\n\n", decode_entities(&caps[1]))
        })
        .to_string();

    // Convert inline code
    text = CODE_RE
        .replace_all(&text, |caps: &regex::Captures| format!("`{}`", decode_entities(&caps[1])))
        .to_string();

    // Convert links
    text = LINK_RE
        .replace_all(&text, |caps: &regex::Captures| {
            let href = &caps[1];
            let link_text = strip_tags(&caps[2]);
            if link_text.is_empty() || href.starts_with('#') || href.starts_with("javascript:") {
                link_text
            } else {
                format!("[{link_text}]({href})")
            }
        })
        .to_string();

    // Convert images
    text = IMG_RE
        .replace_all(&text, |caps: &regex::Captures| format!("![{}]({})", &caps[1], &caps[2]))
        .to_string();

    // Convert emphasis
    text = STRONG_RE
        .replace_all(&text, |caps: &regex::Captures| format!("**{}**", strip_tags(&caps[1])))
        .to_string();
    text = EM_RE
        .replace_all(&text, |caps: &regex::Captures| format!("*{}*", strip_tags(&caps[1])))
        .to_string();

    // Convert list items
    text = LI_RE
        .replace_all(&text, |caps: &regex::Captures| {
            format!("\n- {}", strip_tags(&caps[1]).trim())
        })
        .to_string();

    // Convert blockquotes
    text = BLOCKQUOTE_RE
        .replace_all(&text, |caps: &regex::Captures| {
            let content = strip_tags(&caps[1]);
            let quoted: Vec<String> = content.lines().map(|l| format!("> {l}")).collect();
            format!("\n\n{}\n\n", quoted.join("\n"))
        })
        .to_string();

    // Convert <br> and <hr>
    text = BR_RE.replace_all(&text, "\n").to_string();
    text = HR_RE.replace_all(&text, "\n\n---\n\n").to_string();

    // Convert paragraphs and divs to double newlines
    text = P_DIV_RE.replace_all(&text, "\n\n").to_string();

    // Remove remaining HTML tags
    text = strip_tags(&text);

    // Decode HTML entities
    text = decode_entities(&text);

    // Clean up whitespace
    text = MULTI_NEWLINE_RE.replace_all(&text, "\n\n").to_string();
    text = MULTI_SPACE_RE.replace_all(&text, " ").to_string();

    text.trim().to_string()
}

/// Strip all HTML tags from text.
pub(super) fn strip_tags(html: &str) -> String {
    TAG_STRIP_RE.replace_all(html, "").to_string()
}

/// Decode common HTML entities.
pub(super) fn decode_entities(text: &str) -> String {
    text.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
        .replace("&nbsp;", " ")
        .replace("&#x27;", "'")
        .replace("&#x2F;", "/")
        .replace("&mdash;", "—")
        .replace("&ndash;", "–")
        .replace("&hellip;", "…")
        .replace("&copy;", "©")
        .replace("&reg;", "®")
        .replace("&trade;", "™")
}

#[cfg(test)]
#[path = "html_converter_tests.rs"]
mod tests;
