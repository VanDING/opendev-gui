//! WorkflowService — Approval, Ask-User, and Plan Approval management.
//!
//! Manages pending approvals using oneshot channels for async resolution.

use std::collections::HashMap;
use tokio::sync::{Mutex, oneshot};

/// Approval slot types.
pub struct PendingApproval {
    pub id: String,
    pub tool_name: String,
    pub session_id: Option<String>,
}

pub struct PendingAskUser {
    pub request_id: String,
    pub session_id: Option<String>,
}

pub struct PendingPlanApproval {
    pub request_id: String,
    pub session_id: Option<String>,
}

/// Resolution results.
#[derive(Debug, Clone)]
pub struct ApprovalResult {
    pub approved: bool,
    pub auto_approve: bool,
}

#[derive(Debug, Clone)]
pub struct AskUserResult {
    pub answers: Option<serde_json::Value>,
    pub cancelled: bool,
}

#[derive(Debug, Clone)]
pub struct PlanApprovalResult {
    pub action: String,
    pub feedback: String,
}

/// Internal oneshot slots.
struct ApprovalSlot {
    meta: PendingApproval,
    tx: Option<oneshot::Sender<ApprovalResult>>,
}

struct AskUserSlot {
    meta: PendingAskUser,
    tx: Option<oneshot::Sender<AskUserResult>>,
}

struct PlanApprovalSlot {
    meta: PendingPlanApproval,
    tx: Option<oneshot::Sender<PlanApprovalResult>>,
}

pub struct WorkflowService {
    pending_approvals: Mutex<HashMap<String, ApprovalSlot>>,
    pending_ask_users: Mutex<HashMap<String, AskUserSlot>>,
    pending_plan_approvals: Mutex<HashMap<String, PlanApprovalSlot>>,
}

impl WorkflowService {
    pub fn new() -> Self {
        Self {
            pending_approvals: Mutex::new(HashMap::new()),
            pending_ask_users: Mutex::new(HashMap::new()),
            pending_plan_approvals: Mutex::new(HashMap::new()),
        }
    }

    // ── Approvals ──────────────────────────────────────────────────────

    /// Register a pending tool approval. Returns a receiver for the resolution.
    pub async fn register_approval(
        &self,
        id: String,
        meta: PendingApproval,
    ) -> oneshot::Receiver<ApprovalResult> {
        let (tx, rx) = oneshot::channel();
        self.pending_approvals.lock().await.insert(
            id,
            ApprovalSlot {
                meta,
                tx: Some(tx),
            },
        );
        rx
    }

    /// Resolve a pending approval.
    pub async fn resolve_approval(
        &self,
        id: &str,
        approved: bool,
        auto_approve: bool,
    ) -> Option<PendingApproval> {
        let mut approvals = self.pending_approvals.lock().await;
        if let Some(slot) = approvals.remove(id) {
            if let Some(tx) = slot.tx {
                let _ = tx.send(ApprovalResult {
                    approved,
                    auto_approve,
                });
            }
            Some(slot.meta)
        } else {
            None
        }
    }

    /// Get all pending approvals (for tasks list).
    pub async fn pending_approval_ids(&self) -> Vec<String> {
        self.pending_approvals.lock().await.keys().cloned().collect()
    }

    // ── Ask-User ──────────────────────────────────────────────────────

    /// Register a pending ask-user request.
    pub async fn register_ask_user(
        &self,
        request_id: String,
        meta: PendingAskUser,
    ) -> oneshot::Receiver<AskUserResult> {
        let (tx, rx) = oneshot::channel();
        self.pending_ask_users.lock().await.insert(
            request_id,
            AskUserSlot {
                meta,
                tx: Some(tx),
            },
        );
        rx
    }

    /// Resolve a pending ask-user request.
    pub async fn resolve_ask_user(
        &self,
        request_id: &str,
        answers: Option<serde_json::Value>,
        cancelled: bool,
    ) -> Option<PendingAskUser> {
        let mut ask_users = self.pending_ask_users.lock().await;
        if let Some(slot) = ask_users.remove(request_id) {
            if let Some(tx) = slot.tx {
                let _ = tx.send(AskUserResult { answers, cancelled });
            }
            Some(slot.meta)
        } else {
            None
        }
    }

    // ── Plan Approvals ────────────────────────────────────────────────

    /// Register a pending plan approval request.
    pub async fn register_plan_approval(
        &self,
        request_id: String,
        meta: PendingPlanApproval,
    ) -> oneshot::Receiver<PlanApprovalResult> {
        let (tx, rx) = oneshot::channel();
        self.pending_plan_approvals.lock().await.insert(
            request_id,
            PlanApprovalSlot {
                meta,
                tx: Some(tx),
            },
        );
        rx
    }

    /// Resolve a pending plan approval.
    pub async fn resolve_plan_approval(
        &self,
        request_id: &str,
        action: String,
        feedback: String,
    ) -> Option<PendingPlanApproval> {
        let mut plan_approvals = self.pending_plan_approvals.lock().await;
        if let Some(slot) = plan_approvals.remove(request_id) {
            if let Some(tx) = slot.tx {
                let _ = tx.send(PlanApprovalResult { action, feedback });
            }
            Some(slot.meta)
        } else {
            None
        }
    }

    /// Deny/reject all pending workflows (e.g., on interrupt).
    pub async fn deny_all(&self) {
        {
            let mut approvals = self.pending_approvals.lock().await;
            for (_id, slot) in approvals.iter_mut() {
                if let Some(tx) = slot.tx.take() {
                    let _ = tx.send(ApprovalResult {
                        approved: false,
                        auto_approve: false,
                    });
                }
            }
            approvals.clear();
        }
        {
            let mut ask_users = self.pending_ask_users.lock().await;
            for (_id, slot) in ask_users.iter_mut() {
                if let Some(tx) = slot.tx.take() {
                    let _ = tx.send(AskUserResult {
                        answers: None,
                        cancelled: true,
                    });
                }
            }
            ask_users.clear();
        }
        {
            let mut plan_approvals = self.pending_plan_approvals.lock().await;
            for (_id, slot) in plan_approvals.iter_mut() {
                if let Some(tx) = slot.tx.take() {
                    let _ = tx.send(PlanApprovalResult {
                        action: "reject".to_string(),
                        feedback: "Interrupted".to_string(),
                    });
                }
            }
            plan_approvals.clear();
        }
    }
}
