//! Contrato compartilhado Rust <-> frontend. Espelhado em `src/types.ts`. NÃO alterar sem avisar o mantenedor.
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    pub dir: PathBuf,
}

/// Opção de modelo exibida na GUI. Espelhada em `ModelOption` (src/types.ts).
#[derive(Debug, Clone, Serialize)]
pub struct ModelOption {
    pub provider: String,
    pub id: String,
    pub label: String,
    pub default: bool,
}

/// evento "subagent-start"
#[derive(Debug, Clone, Serialize)]
pub struct SubagentStart {
    pub agent_id: String,
    pub agent_type: String,
    pub description: Option<String>,
    pub prompt: Option<String>,
}
/// evento "subagent-log": uma entrada nova do transcript do subagente.
#[derive(Debug, Clone, Serialize)]
pub struct SubagentLog {
    pub agent_id: String,
    /// "user" | "text" | "tool" | "result"
    pub kind: String,
    pub text: String,
}
/// evento "subagent-model": modelo efetivamente usado pelo subagente (lido do transcript).
#[derive(Debug, Clone, Serialize)]
pub struct SubagentModel {
    pub agent_id: String,
    pub model: String,
}
/// evento "subagent-stop"
#[derive(Debug, Clone, Serialize)]
pub struct SubagentStop {
    pub agent_id: String,
    pub last_message: Option<String>,
}

// ---- terminal principal (PTY). task_id é sempre "main" ----
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

/// evento "usage-update": uso de cota de uma IA (hoje só "claude", lido do statusLine do Claude Code).
/// Janelas ausentes (ex.: plano sem rate_limits) vêm como None.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct UsageUpdate {
    /// "claude" (futuro: outras IAs)
    pub provider: String,
    /// % da janela de 5h (0–100) e epoch (s) do reset
    pub session_pct: Option<f64>,
    pub session_resets_at: Option<i64>,
    /// % da janela semanal (0–100) e epoch (s) do reset
    pub week_pct: Option<f64>,
    pub week_resets_at: Option<i64>,
}
