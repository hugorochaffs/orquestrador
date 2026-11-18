//! Comandos Tauri. Contrato fixo; implementações delegam aos módulos.
use crate::orchestrator::Orchestrator;
use crate::pty::PtyManager;
use crate::types::*;
use crate::{orchestrator, planner};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};

#[tauri::command]
pub fn get_session(session: State<'_, SessionConfig>) -> SessionConfig {
    session.inner().clone()
}

#[tauri::command]
pub async fn start_run(
    app: AppHandle,
    session: State<'_, SessionConfig>,
    pty: State<'_, Arc<PtyManager>>,
    orch: State<'_, Arc<Orchestrator>>,
) -> Result<Plan, String> {
    let session = session.inner().clone();
    let plan = planner::plan(&session).await?;
    planner::validate(&plan)?;
    let _ = app.emit("plan-ready", PlanReady { plan: plan.clone() });
    let (pty, orch) = (pty.inner().clone(), orch.inner().clone());
    let p = plan.clone();
    tauri::async_runtime::spawn(async move {
        orchestrator::run(orch, app, session, p, pty).await;
    });
    Ok(plan)
}

#[tauri::command]
pub fn write_pty(pty: State<'_, Arc<PtyManager>>, id: String, data: String) -> Result<(), String> {
    pty.write(&id, data.as_bytes())
}

#[tauri::command]
pub fn resize_pty(
    pty: State<'_, Arc<PtyManager>>,
    id: String,
    cols: u16,
    rows: u16,
) -> Result<(), String> {
    pty.resize(&id, cols, rows)
}

#[tauri::command]
pub fn resolve_conflict(
    orch: State<'_, Arc<Orchestrator>>,
    task_id: String,
    action: ConflictAction,
) -> Result<(), String> {
    orch.resolve(&task_id, action)
}
