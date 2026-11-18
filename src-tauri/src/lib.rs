pub mod cli;
pub mod detect;
pub mod ipc;
pub mod pty;
pub mod session;
pub mod subagents;
pub mod tools;
pub mod types;
pub mod usage;

use std::sync::Arc;
use types::SessionConfig;

/// Sobe a GUI Tauri com a sessão já montada pelo CLI.
pub fn run(session: SessionConfig) {
    tauri::Builder::default()
        .manage(session)
        .manage(Arc::new(pty::PtyManager::default()))
        .invoke_handler(tauri::generate_handler![
            ipc::get_session,
            ipc::start_main,
            ipc::list_models,
            ipc::write_pty,
            ipc::resize_pty,
            ipc::quit_app
        ])
        .run(tauri::generate_context!())
        .expect("erro ao iniciar o Orchestra");
}
