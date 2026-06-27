//! Workflow commands — DTO mapping only.

use crate::application::AppServices;
use crate::interface::desktop::contract::workflow::*;
use tauri::State;

/// Approve or reject a tool execution.
#[tauri::command]
pub async fn approve_tool(
    services: State<'_, AppServices>,
    req: ApprovalResponse,
) -> Result<WorkflowActionResult, String> {
    let resolved =
        services.workflow.resolve_approval(&req.approval_id, req.approved, req.auto_approve).await;

    match resolved {
        Some(_) => Ok(WorkflowActionResult {
            status: "resolved".to_string(),
            message: Some(format!(
                "Approval {} {}",
                req.approval_id,
                if req.approved { "approved" } else { "rejected" }
            )),
        }),
        None => Ok(WorkflowActionResult {
            status: "not_found".to_string(),
            message: Some(format!("Approval {} not found", req.approval_id)),
        }),
    }
}

/// Respond to an ask-user request.
#[tauri::command]
pub async fn respond_to_ask(
    services: State<'_, AppServices>,
    req: AskUserResponse,
) -> Result<WorkflowActionResult, String> {
    let resolved =
        services.workflow.resolve_ask_user(&req.request_id, req.answers, req.cancelled).await;

    match resolved {
        Some(_) => Ok(WorkflowActionResult {
            status: "resolved".to_string(),
            message: Some(format!("Ask-user {} resolved", req.request_id)),
        }),
        None => Ok(WorkflowActionResult {
            status: "not_found".to_string(),
            message: Some(format!("Ask-user request {} not found", req.request_id)),
        }),
    }
}

/// Respond to a plan approval request.
#[tauri::command]
pub async fn respond_to_plan(
    services: State<'_, AppServices>,
    req: PlanApprovalResponse,
) -> Result<WorkflowActionResult, String> {
    let action = req.action.clone();
    let resolved =
        services.workflow.resolve_plan_approval(&req.request_id, action, req.feedback).await;

    match resolved {
        Some(_) => Ok(WorkflowActionResult {
            status: "resolved".to_string(),
            message: Some(format!(
                "Plan approval {} resolved with action: {}",
                req.request_id, req.action
            )),
        }),
        None => Ok(WorkflowActionResult {
            status: "not_found".to_string(),
            message: Some(format!("Plan approval {} not found", req.request_id)),
        }),
    }
}
