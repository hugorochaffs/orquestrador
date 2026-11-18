//! STUB (agente B): PtyManager com portable-pty.
use std::path::Path;
use tauri::AppHandle;

#[derive(Default)]
pub struct PtyManager {}

impl PtyManager {
    /// Spawna `argv` (sem shell) em `cwd`. Emite "pty-output" e "pty-exit" (ver types.rs).
    /// `on_exit(code)` é chamado uma vez quando o processo termina. Retorna o pty_id.
    pub fn spawn(
        &self,
        _app: &AppHandle,
        _task_id: &str,
        _argv: Vec<String>,
        _cwd: &Path,
        _on_exit: Box<dyn FnOnce(i32) + Send + 'static>,
    ) -> Result<String, String> {
        Err("não implementado".into())
    }
    pub fn write(&self, _id: &str, _data: &[u8]) -> Result<(), String> {
        Err("não implementado".into())
    }
    pub fn resize(&self, _id: &str, _cols: u16, _rows: u16) -> Result<(), String> {
        Err("não implementado".into())
    }
    pub fn kill(&self, _id: &str) -> Result<(), String> {
        Err("não implementado".into())
    }
}
