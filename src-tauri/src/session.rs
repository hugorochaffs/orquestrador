//! STUB (agente G1): sobe o `claude` interativo no PTY principal com hooks que alimentam o arquivo de eventos.
use crate::pty::PtyManager;
use crate::types::SessionConfig;
use std::sync::Arc;
use tauri::AppHandle;

/// Spawna o claude interativo (task_id "main"), chama `crate::subagents::spawn_watcher` e devolve o pty_id.
pub fn start(_app: &AppHandle, _s: &SessionConfig, _pty: &Arc<PtyManager>) -> Result<String, String> {
    Err("não implementado".into())
}
