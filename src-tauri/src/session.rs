//! Sobe o `claude` interativo no PTY principal com hooks que alimentam o arquivo de eventos.
use crate::pty::PtyManager;
use crate::types::SessionConfig;
use serde_json::{json, Value};
use std::io::Write;
use std::path::Path;
use std::sync::Arc;
use tauri::AppHandle;

/// Quoting POSIX de shell com aspas simples.
pub fn shell_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', r"'\''"))
}

/// JSON de `--settings` com hooks que chamam `<exe> hook <events_file>`.
pub fn hook_settings(exe: &Path, events_file: &Path) -> String {
    let cmd = format!(
        "{} hook {}",
        shell_quote(&exe.to_string_lossy()),
        shell_quote(&events_file.to_string_lossy())
    );
    let hooks = json!([{ "type": "command", "command": cmd }]);
    let plain = json!([{ "hooks": hooks }]);
    let v: Value = json!({ "hooks": {
        "SubagentStart": plain,
        "SubagentStop": plain,
        "PreToolUse": [{ "matcher": "Agent|Task", "hooks": hooks }],
    }});
    v.to_string()
}

/// Implementação do subcomando `hook`: acrescenta o JSON compactado como uma linha. Nunca falha.
pub fn run_hook(events_file: &Path, input: &str) {
    let Ok(v) = serde_json::from_str::<Value>(input) else { return };
    let mut line = v.to_string();
    line.push('\n');
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(events_file) {
        let _ = f.write_all(line.as_bytes());
    }
}

fn exclude_orchestra(dir: &Path) {
    let Ok(o) = std::process::Command::new("git")
        .arg("-C").arg(dir).args(["rev-parse", "--git-path", "info/exclude"]).output()
    else { return };
    if !o.status.success() {
        return;
    }
    let rel = String::from_utf8_lossy(&o.stdout).trim().to_string();
    let p = dir.join(rel);
    if std::fs::read_to_string(&p).is_ok_and(|s| s.lines().any(|l| l.trim() == ".orchestra/")) {
        return;
    }
    if let Some(parent) = p.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&p) {
        let _ = f.write_all(b"\n.orchestra/\n");
    }
}

/// Spawna o claude interativo (task_id "main"), chama `crate::subagents::spawn_watcher` e devolve o pty_id.
pub fn start(app: &AppHandle, session: &SessionConfig, pty: &Arc<PtyManager>) -> Result<String, String> {
    let odir = session.dir.join(".orchestra");
    std::fs::create_dir_all(&odir).map_err(|e| format!("falha ao criar .orchestra: {e}"))?;
    let events = odir.join("events.jsonl");
    std::fs::write(&events, b"").map_err(|e| format!("falha ao criar events.jsonl: {e}"))?;
    exclude_orchestra(&session.dir);

    let exe = std::env::current_exe().map_err(|e| format!("current_exe: {e}"))?;
    let settings = hook_settings(&exe, &events);
    let argv = crate::tools::claude_argv(session.model.as_deref(), &settings);

    crate::subagents::spawn_watcher(app.clone(), events);
    pty.spawn(app, "main", argv, &session.dir, Box::new(|_| {}))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn quoting() {
        assert_eq!(shell_quote("a b"), "'a b'");
        assert_eq!(shell_quote("it's"), r"'it'\''s'");
    }

    #[test]
    fn settings_json() {
        let s = hook_settings(Path::new("/opt/my app/orchestra"), Path::new("/p/e.jsonl"));
        let v: Value = serde_json::from_str(&s).unwrap();
        let cmd = "'/opt/my app/orchestra' hook '/p/e.jsonl'";
        assert_eq!(v["hooks"]["SubagentStart"][0]["hooks"][0]["command"], cmd);
        assert_eq!(v["hooks"]["SubagentStop"][0]["hooks"][0]["type"], "command");
        assert_eq!(v["hooks"]["PreToolUse"][0]["matcher"], "Agent|Task");
        assert_eq!(v["hooks"]["PreToolUse"][0]["hooks"][0]["command"], cmd);
    }

    #[test]
    fn hook_appends_lines() {
        let f: PathBuf = std::env::temp_dir().join(format!("orch_hook_{}.jsonl", std::process::id()));
        let _ = std::fs::remove_file(&f);
        run_hook(&f, "{\n \"a\": 1\n}");
        run_hook(&f, "lixo");
        run_hook(&f, "{\"b\":2}");
        assert_eq!(std::fs::read_to_string(&f).unwrap(), "{\"a\":1}\n{\"b\":2}\n");
        let _ = std::fs::remove_file(&f);
    }
}
