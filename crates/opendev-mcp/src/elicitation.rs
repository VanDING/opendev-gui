//! MCP Elicitation — collect user input for unresolved parameters.
//!
//! When an MCP tool call has missing or ambiguous parameters, the
//! elicitation system formats a request for the missing information
//! and collects the user's response.
//!
//! Supports two modes:
//! - **Form mode**: Presents a structured form based on JSON Schema.
//! - **URL mode**: Opens a browser to a URL for interactive data collection.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use uuid::Uuid;

/// How to elicit information from the user.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElicitMode {
    /// Present a structured form (default).
    Form,
    /// Open a URL in the browser.
    Url,
}

/// A request for user input.
#[derive(Debug, Clone)]
pub struct ElicitRequest {
    /// How to collect the information.
    pub mode: ElicitMode,
    /// JSON Schema describing the required parameters.
    pub schema: serde_json::Value,
    /// Unique identifier for this elicitation session.
    pub elicitation_id: String,
    /// Human-readable prompt describing what's needed.
    pub prompt: String,
    /// Optional URL for URL mode.
    pub url: Option<String>,
}

/// Result of a successful elicitation.
#[derive(Debug, Clone)]
pub struct ElicitResult {
    /// The elicitation request this is a response to.
    pub elicitation_id: String,
    /// The collected parameter values (key-value pairs).
    pub values: HashMap<String, serde_json::Value>,
}

/// Handler for managing elicitation requests and responses.
///
/// Stores pending elicitation requests and provides a mechanism to
/// submit results back.
#[derive(Debug)]
pub struct ElicitationHandler {
    /// Pending elicitation requests, keyed by ID.
    pending: Arc<Mutex<HashMap<String, PendingElicitation>>>,
}

/// Internal state for a pending elicitation.
#[derive(Debug)]
struct PendingElicitation {
    /// The original request.
    request: ElicitRequest,
    /// Sender for submitting the result.
    result_tx: tokio::sync::oneshot::Sender<ElicitResult>,
}

impl ElicitationHandler {
    /// Create a new elicitation handler.
    pub fn new() -> Self {
        Self {
            pending: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Request user input for a given JSON Schema.
    ///
    /// Returns an `ElicitRequest` describing what's needed and a receiver
    /// that will deliver the result when the user responds.
    pub fn request_input(
        &self,
        schema: serde_json::Value,
        prompt: impl Into<String>,
        mode: ElicitMode,
        url: Option<String>,
    ) -> (ElicitRequest, tokio::sync::oneshot::Receiver<ElicitResult>) {
        let id = Uuid::new_v4().to_string();
        let (tx, rx) = tokio::sync::oneshot::channel();

        let request = ElicitRequest {
            mode,
            schema,
            elicitation_id: id.clone(),
            prompt: prompt.into(),
            url,
        };

        let pending = PendingElicitation {
            request: request.clone(),
            result_tx: tx,
        };

        // Store in pending map
        let mut map = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current()
                .block_on(async { self.pending.lock().await })
        });
        map.insert(id.clone(), pending);

        (request, rx)
    }

    /// Submit the result for a pending elicitation request.
    ///
    /// Returns `None` if the elicitation ID is unknown or already completed.
    pub fn submit_result(
        &self,
        elicitation_id: &str,
        values: HashMap<String, serde_json::Value>,
    ) -> Option<()> {
        let mut map = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current()
                .block_on(async { self.pending.lock().await })
        });

        match map.remove(elicitation_id) {
            Some(pending) => {
                let result = ElicitResult {
                    elicitation_id: elicitation_id.to_string(),
                    values,
                };
                let _ = pending.result_tx.send(result);
                Some(())
            }
            None => None,
        }
    }

    /// Get a list of all pending elicitation requests.
    pub fn pending_requests(&self) -> Vec<ElicitRequest> {
        let map = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current()
                .block_on(async { self.pending.lock().await })
        });
        map.values().map(|p| p.request.clone()).collect()
    }

    /// Cancel a pending elicitation request.
    pub fn cancel(&self, elicitation_id: &str) -> bool {
        let mut map = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current()
                .block_on(async { self.pending.lock().await })
        });
        map.remove(elicitation_id).is_some()
    }

    /// Process a JSON Schema and extract form field definitions.
    ///
    /// Returns a list of field descriptors suitable for rendering as a form.
    pub fn schema_to_form_fields(schema: &serde_json::Value) -> Vec<FormField> {
        let mut fields = Vec::new();

        let properties = match schema.get("properties").and_then(|p| p.as_object()) {
            Some(props) => props,
            None => return fields,
        };

        let required = schema
            .get("required")
            .and_then(|r| r.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect::<std::collections::HashSet<_>>()
            })
            .unwrap_or_default();

        for (name, prop_schema) in properties {
            let prop_type = prop_schema
                .get("type")
                .and_then(|t| t.as_str())
                .unwrap_or("string")
                .to_string();

            let description = prop_schema
                .get("description")
                .and_then(|d| d.as_str())
                .unwrap_or("")
                .to_string();

            let default = prop_schema.get("default").cloned();

            let enum_values = prop_schema
                .get("enum")
                .and_then(|e| e.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                });

            fields.push(FormField {
                name: name.clone(),
                field_type: prop_type,
                description,
                required: required.contains(name),
                default,
                enum_values,
            });
        }

        fields
    }
}

impl Default for ElicitationHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// A single form field derived from JSON Schema.
#[derive(Debug, Clone)]
pub struct FormField {
    /// Field name (property key).
    pub name: String,
    /// JSON Schema type (string, number, boolean, etc.).
    pub field_type: String,
    /// Human-readable description.
    pub description: String,
    /// Whether the field is required.
    pub required: bool,
    /// Default value, if specified.
    pub default: Option<serde_json::Value>,
    /// Enum values, if specified.
    pub enum_values: Option<Vec<String>>,
}

/// Open a URL in the default browser.
///
/// Used for URL-mode elicitation where the user completes a form
/// or authorizes via their browser.
pub fn open_url_in_browser(url: &str) -> Result<(), String> {
    open::that(url).map_err(|e| format!("Failed to open browser: {e}"))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_elicit_request_form() {
        let handler = ElicitationHandler::new();
        let schema = serde_json::json!({
            "type": "object",
            "properties": {
                "name": {"type": "string", "description": "Your name"},
                "age": {"type": "integer", "description": "Your age"}
            },
            "required": ["name"]
        });

        let (request, _rx) = handler.request_input(schema, "Please provide your info", ElicitMode::Form, None);

        assert_eq!(request.mode, ElicitMode::Form);
        assert!(request.prompt.contains("Please provide your info"));
        assert!(!request.elicitation_id.is_empty());
        assert!(request.url.is_none());
    }

    #[test]
    fn test_elicit_request_url() {
        let handler = ElicitationHandler::new();
        let schema = serde_json::json!({"type": "object", "properties": {}});

        let (request, _rx) = handler.request_input(
            schema,
            "Authorize in browser",
            ElicitMode::Url,
            Some("https://example.com/auth".to_string()),
        );

        assert_eq!(request.mode, ElicitMode::Url);
        assert_eq!(request.url.as_deref(), Some("https://example.com/auth"));
    }

    #[test]
    fn test_submit_result() {
        let handler = ElicitationHandler::new();
        let schema = serde_json::json!({"type": "object", "properties": {}});
        let (request, rx) = handler.request_input(schema, "test", ElicitMode::Form, None);

        let mut values = HashMap::new();
        values.insert("answer".to_string(), serde_json::json!(42));

        // Submit result
        assert!(handler.submit_result(&request.elicitation_id, values.clone()).is_some());

        // Should receive the result
        let result = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current()
                .block_on(async { rx.await })
        });
        let result = result.expect("should receive result");
        assert_eq!(result.values["answer"], serde_json::json!(42));
    }

    #[test]
    fn test_submit_unknown_id() {
        let handler = ElicitationHandler::new();
        assert!(handler.submit_result("nonexistent", HashMap::new()).is_none());
    }

    #[test]
    fn test_cancel_pending() {
        let handler = ElicitationHandler::new();
        let schema = serde_json::json!({"type": "object", "properties": {}});
        let (request, _rx) = handler.request_input(schema, "test", ElicitMode::Form, None);

        assert!(handler.cancel(&request.elicitation_id));
        assert!(!handler.cancel(&request.elicitation_id)); // second cancel fails
    }

    #[test]
    fn test_pending_requests() {
        let handler = ElicitationHandler::new();
        let schema = serde_json::json!({"type": "object", "properties": {}});
        let (_r1, _) = handler.request_input(schema.clone(), "first", ElicitMode::Form, None);
        let (_r2, _) = handler.request_input(schema, "second", ElicitMode::Form, None);

        assert_eq!(handler.pending_requests().len(), 2);
    }

    #[test]
    fn test_schema_to_form_fields() {
        let schema = serde_json::json!({
            "type": "object",
            "properties": {
                "username": {
                    "type": "string",
                    "description": "Your username",
                    "default": "guest"
                },
                "role": {
                    "type": "string",
                    "enum": ["admin", "user", "viewer"]
                },
                "count": {
                    "type": "integer",
                    "description": "Number of items"
                }
            },
            "required": ["username", "role"]
        });

        let fields = ElicitationHandler::schema_to_form_fields(&schema);
        assert_eq!(fields.len(), 3);

        let username = fields.iter().find(|f| f.name == "username").unwrap();
        assert_eq!(username.field_type, "string");
        assert!(username.required);
        assert_eq!(username.default, Some(serde_json::json!("guest")));

        let role = fields.iter().find(|f| f.name == "role").unwrap();
        assert!(role.required);
        assert_eq!(role.enum_values, Some(vec!["admin".into(), "user".into(), "viewer".into()]));

        let count = fields.iter().find(|f| f.name == "count").unwrap();
        assert!(!count.required);
        assert_eq!(count.field_type, "integer");
    }

    #[test]
    fn test_schema_to_form_fields_empty() {
        let fields = ElicitationHandler::schema_to_form_fields(&serde_json::json!({"type": "object"}));
        assert!(fields.is_empty());
    }
}
