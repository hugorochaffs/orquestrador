//! STUB (agente G2): observa o arquivo de eventos dos hooks e os transcripts dos subagentes.
use std::path::PathBuf;
use tauri::AppHandle;

/// `events_file`: JSONL em que cada linha é o JSON bruto de um hook do Claude Code
/// (SubagentStart, SubagentStop, PreToolUse). Emite "subagent-start", "subagent-log", "subagent-stop".
pub fn spawn_watcher(_app: AppHandle, _events_file: PathBuf) {}
