//! STUB (agente C): agendador do DAG + merge automático.
use crate::pty::PtyManager;
use crate::types::*;
use std::sync::Arc;
use tauri::AppHandle;

#[derive(Default)]
pub struct Orchestrator {}

impl Orchestrator {
    pub fn resolve(&self, _task_id: &str, _action: ConflictAction) -> Result<(), String> {
        Err("não implementado".into())
    }
}

/// Executa o plano inteiro; emite "task-status" (ver types.rs).
pub async fn run(_o: Arc<Orchestrator>, _app: AppHandle, _s: SessionConfig, _p: Plan, _pty: Arc<PtyManager>) {}
