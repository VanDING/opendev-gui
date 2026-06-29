use super::*;

#[test]
fn test_parse_manifest() {
    let json = r#"{
        "name": "my_tool",
        "description": "A custom tool",
        "command": "./run.sh",
        "parameters": {
            "type": "object",
            "properties": {
                "input": { "type": "string" }
            },
            "required": ["input"]
        },
        "timeout_secs": 60
    }"#;

    let manifest: CustomToolManifest = serde_json::from_str(json).unwrap();
    assert_eq!(manifest.name, "my_tool");
    assert_eq!(manifest.description, "A custom tool");
    assert_eq!(manifest.command, "./run.sh");
    assert_eq!(manifest.timeout_secs, 60);
}

#[test]
fn test_parse_manifest_defaults() {
    let json = r#"{
        "name": "simple",
        "description": "Simple tool",
        "command": "echo"
    }"#;

    let manifest: CustomToolManifest = serde_json::from_str(json).unwrap();
    assert_eq!(manifest.timeout_secs, 30);
    assert!(manifest.parameters.is_object());
}

#[test]
fn test_discover_empty_dir() {
    let tmp = tempfile::TempDir::new().unwrap();
    let tools = discover_custom_tools(tmp.path());
    assert!(tools.is_empty());
}

#[test]
fn test_discover_finds_manifests() {
    let tmp = tempfile::TempDir::new().unwrap();
    let tool_dir = tmp.path().join(".opendev").join("tools");
    std::fs::create_dir_all(&tool_dir).unwrap();

    // Create a manifest
    let manifest = r#"{
        "name": "test_tool",
        "description": "Test",
        "command": "./test.sh"
    }"#;
    std::fs::write(tool_dir.join("test.tool.json"), manifest).unwrap();

    // Create a non-manifest file (should be ignored)
    std::fs::write(tool_dir.join("readme.md"), "ignore me").unwrap();

    let tools = discover_custom_tools(tmp.path());
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0].name(), "test_tool");
}

#[test]
fn test_discover_deduplicates() {
    let tmp = tempfile::TempDir::new().unwrap();

    // Same tool name in both directories
    let dir1 = tmp.path().join(".opendev").join("tools");
    let dir2 = tmp.path().join(".opencode").join("tool");
    std::fs::create_dir_all(&dir1).unwrap();
    std::fs::create_dir_all(&dir2).unwrap();

    let manifest = r#"{"name": "dup", "description": "Dup", "command": "./x.sh"}"#;
    std::fs::write(dir1.join("dup.tool.json"), manifest).unwrap();
    std::fs::write(dir2.join("dup.tool.json"), manifest).unwrap();

    let tools = discover_custom_tools(tmp.path());
    assert_eq!(tools.len(), 1, "Duplicate tool names should be deduplicated");
}

#[test]
fn test_resolve_command_relative() {
    let manifest = CustomToolManifest {
        name: "t".into(),
        description: "t".into(),
        command: "./run.sh".into(),
        parameters: default_params_schema(),
        timeout_secs: 30,
    };
    let tool = CustomTool::new(manifest, PathBuf::from("/project/.opendev/tools"));
    assert_eq!(tool.resolve_command(), PathBuf::from("/project/.opendev/tools/run.sh"));
}

#[test]
fn test_resolve_command_absolute() {
    let manifest = CustomToolManifest {
        name: "t".into(),
        description: "t".into(),
        command: "/usr/bin/my-tool".into(),
        parameters: default_params_schema(),
        timeout_secs: 30,
    };
    let tool = CustomTool::new(manifest, PathBuf::from("/project/.opendev/tools"));
    assert_eq!(tool.resolve_command(), PathBuf::from("/usr/bin/my-tool"));
}

#[tokio::test]
async fn test_execute_missing_command() {
    let manifest = CustomToolManifest {
        name: "missing".into(),
        description: "Missing".into(),
        command: "./nonexistent.sh".into(),
        parameters: default_params_schema(),
        timeout_secs: 5,
    };
    let tmp = tempfile::TempDir::new().unwrap();
    let tool = CustomTool::new(manifest, tmp.path().to_path_buf());
    let ctx = ToolContext::new(tmp.path());
    let result = tool.execute(HashMap::new(), &ctx).await;
    assert!(!result.success);
    assert!(result.error.unwrap().contains("not found"));
}

#[cfg(unix)]
#[tokio::test]
async fn test_execute_simple_command() {
    let tmp = tempfile::TempDir::new().unwrap();
    let script_path = tmp.path().join("echo.sh");
    std::fs::write(&script_path, "#!/bin/sh\necho \"hello from custom tool\"").unwrap();

    // Make executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&script_path, std::fs::Permissions::from_mode(0o755)).unwrap();
    }

    let manifest = CustomToolManifest {
        name: "echo_tool".into(),
        description: "Echo".into(),
        command: "./echo.sh".into(),
        parameters: default_params_schema(),
        timeout_secs: 5,
    };
    let tool = CustomTool::new(manifest, tmp.path().to_path_buf());
    let ctx = ToolContext::new(tmp.path());
    let result = tool.execute(HashMap::new(), &ctx).await;
    assert!(result.success, "Should succeed: {:?}", result.error);
    assert!(result.output.unwrap().contains("hello from custom tool"));
}

#[cfg(unix)]
#[tokio::test]
async fn test_execute_failing_command() {
    let tmp = tempfile::TempDir::new().unwrap();
    let script_path = tmp.path().join("fail.sh");
    std::fs::write(&script_path, "#!/bin/sh\necho 'error msg' >&2\nexit 1").unwrap();

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&script_path, std::fs::Permissions::from_mode(0o755)).unwrap();
    }

    let manifest = CustomToolManifest {
        name: "fail_tool".into(),
        description: "Fail".into(),
        command: "./fail.sh".into(),
        parameters: default_params_schema(),
        timeout_secs: 5,
    };
    let tool = CustomTool::new(manifest, tmp.path().to_path_buf());
    let ctx = ToolContext::new(tmp.path());
    let result = tool.execute(HashMap::new(), &ctx).await;
    assert!(!result.success);
    assert!(result.error.unwrap().contains("error msg"));
}

// ── Validation tests ──

#[test]
fn validate_valid_tool() {
    let manifest = CustomToolManifest {
        name: "test-tool".into(),
        description: "A test tool".into(),
        command: "./script.sh".into(),
        parameters: serde_json::json!({"type": "object", "properties": {}}),
        timeout_secs: 30,
    };
    assert!(validate_custom_tool(&manifest).is_ok());
}

#[test]
fn validate_empty_name() {
    let manifest = CustomToolManifest {
        name: "".into(),
        description: "".into(),
        command: "echo hello".into(),
        parameters: serde_json::json!({"type": "object", "properties": {}}),
        timeout_secs: 30,
    };
    let result = validate_custom_tool(&manifest);
    assert!(result.is_err());
    assert!(result.unwrap_err().iter().any(|e| e.contains("empty")));
}

#[test]
fn validate_name_with_spaces() {
    let manifest = CustomToolManifest {
        name: "my tool".into(),
        description: "".into(),
        command: "echo".into(),
        parameters: serde_json::json!({"type": "object", "properties": {}}),
        timeout_secs: 30,
    };
    let result = validate_custom_tool(&manifest);
    assert!(result.is_err());
}

#[test]
fn validate_shell_injection_in_command() {
    let manifest = CustomToolManifest {
        name: "bad-tool".into(),
        description: "".into(),
        command: "echo; rm -rf /".into(),
        parameters: serde_json::json!({"type": "object", "properties": {}}),
        timeout_secs: 30,
    };
    let result = validate_custom_tool(&manifest);
    assert!(result.is_err());
    assert!(result.unwrap_err().iter().any(|e| e.contains("';'")));
}

#[test]
fn validate_empty_command() {
    let manifest = CustomToolManifest {
        name: "empty-tool".into(),
        description: "".into(),
        command: "".into(),
        parameters: serde_json::json!({"type": "object", "properties": {}}),
        timeout_secs: 30,
    };
    let result = validate_custom_tool(&manifest);
    assert!(result.is_err());
}

#[test]
fn validate_invalid_schema() {
    let manifest = CustomToolManifest {
        name: "bad-schema".into(),
        description: "".into(),
        command: "echo".into(),
        parameters: serde_json::json!("string"), // Not an object
        timeout_secs: 30,
    };
    let result = validate_custom_tool(&manifest);
    assert!(result.is_err());
}

// ── Parameter substitution tests ──

#[test]
fn substitute_single_param() {
    let mut args = std::collections::HashMap::new();
    args.insert("file".to_string(), serde_json::json!("main.rs"));
    let result = substitute_params("process {file}", &args);
    assert_eq!(result, "process main.rs");
}

#[test]
fn substitute_multiple_params() {
    let mut args = std::collections::HashMap::new();
    args.insert("from".to_string(), serde_json::json!("src"));
    args.insert("to".to_string(), serde_json::json!("dst"));
    let result = substitute_params("copy {from} -> {to}", &args);
    assert_eq!(result, "copy src -> dst");
}

#[test]
fn substitute_unknown_param_left_as_is() {
    let mut args = std::collections::HashMap::new();
    args.insert("known".to_string(), serde_json::json!("value"));
    let result = substitute_params("process {known} and {unknown}", &args);
    assert_eq!(result, "process value and {unknown}");
}

#[test]
fn substitute_non_string_param() {
    let mut args = std::collections::HashMap::new();
    args.insert("count".to_string(), serde_json::json!(42));
    let result = substitute_params("run {count} times", &args);
    assert_eq!(result, "run 42 times");
}

// ── Output parsing tests ──

#[test]
fn parse_output_json_object() {
    let stdout = r#"{"status": "ok", "result": "done"}"#;
    let (display, parsed) = parse_tool_output(stdout);
    assert!(parsed.is_some());
    assert!(display.contains("status"));
    assert!(display.contains("ok"));
}

#[test]
fn parse_output_plain_text() {
    let stdout = "Task completed successfully";
    let (display, parsed) = parse_tool_output(stdout);
    assert!(parsed.is_none());
    assert_eq!(display, "Task completed successfully");
}

#[test]
fn parse_output_empty() {
    let (display, parsed) = parse_tool_output("");
    assert!(parsed.is_none());
    assert!(display.is_empty());
}

// ── Security verification tests ──

#[test]
fn verify_security_valid_command() {
    assert!(verify_tool_security("./script.sh").is_ok());
    assert!(verify_tool_security("python3 test.py").is_ok());
}

#[test]
fn verify_security_rejects_dangerous() {
    assert!(verify_tool_security("echo; ls").is_err());
    assert!(verify_tool_security("cmd | grep x").is_err());
    assert!(verify_tool_security("$SHELL").is_err());
    assert!(verify_tool_security("`id`").is_err());
}
