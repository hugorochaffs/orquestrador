//! Comandos Tauri. Contrato fixo; implementações delegam aos módulos.
use crate::pty::PtyManager;
use crate::types::*;
use std::sync::Arc;
use tauri::{AppHandle, State};

#[tauri::command]
pub fn get_session(session: State<'_, SessionConfig>) -> SessionConfig {
    session.inner().clone()
}

/// Sobe o Claude Code interativo (terminal principal) e o observador de subagentes. Retorna o pty_id.
#[tauri::command]
pub fn start_main(
    app: AppHandle,
    session: State<'_, SessionConfig>,
    pty: State<'_, Arc<PtyManager>>,
    model: Option<String>,
) -> Result<String, String> {
    crate::session::start(&app, session.inner(), model.as_deref(), pty.inner())
}

/// Modelos disponíveis para escolha na GUI.
#[tauri::command]
pub fn list_models() -> Vec<ModelOption> {
    crate::tools::models()
}

#[tauri::command]
pub fn write_pty(pty: State<'_, Arc<PtyManager>>, id: String, data: String) -> Result<(), String> {
    pty.write(&id, data.as_bytes())
}

#[tauri::command]
pub fn resize_pty(pty: State<'_, Arc<PtyManager>>, id: String, cols: u16, rows: u16) -> Result<(), String> {
    pty.resize(&id, cols, rows)
}
