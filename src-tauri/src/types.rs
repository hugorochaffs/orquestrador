//! Contrato compartilhado Rust <-> frontend. Espelhado em `src/types.ts`. NÃO alterar sem avisar o mantenedor.
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    pub dir: PathBuf,
    /// id da ferramenta de execução: "claude" | "aider" | "cline"
    pub tool: String,
    pub provider: String,
    pub planner_model: String,
    pub goal: String,
    pub mock_planner: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub prompt: String,
    #[serde(default)]
    pub depends_on: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    pub plan_summary: String,
    pub tasks: Vec<Task>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Pending,
    Running,
    Merging,
    Resolving,
    Done,
    Failed,
    NeedsUser,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConflictAction {
    Retry,
    Abort,
    ManualDone,
}

// ---- payloads de eventos Tauri ----
/// evento "plan-ready"
#[derive(Debug, Clone, Serialize)]
pub struct PlanReady {
    pub plan: Plan,
}
/// evento "task-status"
#[derive(Debug, Clone, Serialize)]
pub struct TaskStatusEvent {
    pub task_id: String,
    pub status: TaskStatus,
    /// pty associado (quando houver) para o nó se conectar ao terminal
    pub pty_id: Option<String>,
}
/// evento "pty-output" (data em base64)
#[derive(Debug, Clone, Serialize)]
pub struct PtyOutput {
    pub pty_id: String,
    pub task_id: String,
    pub data: String,
}
/// evento "pty-exit"
#[derive(Debug, Clone, Serialize)]
pub struct PtyExit {
    pub pty_id: String,
    pub task_id: String,
    pub code: i32,
}
