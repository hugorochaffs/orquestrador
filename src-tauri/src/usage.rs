//! STUB (agente J): observa o arquivo de uso gravado pelo subcomando `statusline` e emite "usage-update".
use std::path::PathBuf;
use tauri::AppHandle;

/// `usage_file`: JSON único (sobrescrito a cada atualização) com o payload bruto do statusLine do Claude Code.
pub fn spawn_watcher(_app: AppHandle, _usage_file: PathBuf) {}
