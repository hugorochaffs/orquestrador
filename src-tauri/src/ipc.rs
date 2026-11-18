//! Comandos Tauri. Contrato fixo; implementações delegam aos módulos.
use crate::pty::PtyManager;
use crate::session::Sessions;
use crate::types::*;
use std::sync::Arc;
use tauri::{AppHandle, State};

#[tauri::command]
pub fn get_session(session: State<'_, SessionConfig>) -> SessionConfig {
    session.inner().clone()
}

/// Sobe o Claude Code interativo da aba `tab_id` e o observador de subagentes dela. Retorna o pty_id.
/// `browser`: liga o Claude in Chrome (`--chrome`); a escolha é persistida no config.json.
#[tauri::command]
pub fn start_main(
    app: AppHandle,
    session: State<'_, SessionConfig>,
    pty: State<'_, Arc<PtyManager>>,
    sessions: State<'_, Sessions>,
    tab_id: String,
    model: Option<String>,
    browser: Option<bool>,
) -> Result<String, String> {
    let browser = browser.unwrap_or(false);
    let id = crate::session::start(&app, session.inner(), &tab_id, model.as_deref(), browser, pty.inner(), sessions.inner())?;
    crate::config::Config::save_browser("claude", browser);
    Ok(id)
}

/// Fecha a sessão da aba: mata o PTY e para o watcher. Idempotente.
#[tauri::command]
pub fn close_tab(pty: State<'_, Arc<PtyManager>>, sessions: State<'_, Sessions>, tab_id: String) {
    sessions.close(&tab_id, pty.inner());
}

/// Opções de navegador por IA (com a última escolha salva).
#[tauri::command]
pub fn list_browser_options() -> Vec<BrowserOption> {
    crate::tools::browser_options(&crate::config::Config::load())
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

/// Fecha o app (Ctrl+C na tela inicial).
#[tauri::command]
pub fn quit_app(app: AppHandle) {
    app.exit(0);
}
