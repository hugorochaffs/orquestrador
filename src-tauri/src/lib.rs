pub mod cli;
pub mod detect;
pub mod git;
pub mod ipc;
pub mod orchestrator;
pub mod planner;
pub mod pty;
pub mod tools;
pub mod types;

use std::sync::Arc;
use types::SessionConfig;

/// Sobe a GUI Tauri depois que o wizard montou a sessão.
pub fn run(session: SessionConfig) {
    tauri::Builder::default()
        .manage(session)
        .manage(Arc::new(pty::PtyManager::default()))
        .manage(Arc::new(orchestrator::Orchestrator::default()))
        .invoke_handler(tauri::generate_handler![
            ipc::get_session,
            ipc::start_run,
            ipc::write_pty,
            ipc::resize_pty,
            ipc::resolve_conflict
        ])
        .run(tauri::generate_context!())
        .expect("erro ao iniciar o Orchestra");
}
