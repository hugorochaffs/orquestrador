//! Sobe o `claude` interativo no PTY principal com hooks que alimentam o arquivo de eventos.
use crate::pty::PtyManager;
use crate::types::SessionConfig;
use serde_json::{json, Value};
use std::io::Write;
use std::path::Path;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, Once};
use tauri::AppHandle;

/// Quoting POSIX de shell com aspas simples.
pub fn shell_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', r"'\''"))
}

/// JSON de `--settings` com hooks que chamam `<exe> hook <events_file>` e statusLine que chama
/// `<exe> statusline <usage_file>`.
pub fn hook_settings(exe: &Path, events_file: &Path, usage_file: &Path, refresh_secs: u64) -> String {
    let cmd = format!(
        "{} hook {}",
        shell_quote(&exe.to_string_lossy()),
        shell_quote(&events_file.to_string_lossy())
    );
    let status_cmd = format!(
        "{} statusline {}",
        shell_quote(&exe.to_string_lossy()),
        shell_quote(&usage_file.to_string_lossy())
    );
    let hooks = json!([{ "type": "command", "command": cmd }]);
    let plain = json!([{ "hooks": hooks }]);
    let v: Value = json!({ "statusLine": { "type": "command", "command": status_cmd, "refreshInterval": refresh_secs }, "hooks": {
        "SubagentStart": plain,
        "SubagentStop": plain,
        "PreToolUse": [
            { "matcher": "Agent|Task", "hooks": hooks },
            // Claude in Chrome: avisa o Orchestra que o navegador externo está em uso (aba marcador).
            { "matcher": "mcp__claude-in-chrome__.*", "hooks": hooks },
        ],
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

/// Implementação do subcomando `statusline`: valida o JSON e grava atomicamente (tmp + rename). Nunca falha.
pub fn run_statusline(usage_file: &Path, input: &str) {
    if serde_json::from_str::<Value>(input).is_err() {
        return;
    }
    let mut tmp = usage_file.as_os_str().to_owned();
    tmp.push(format!(".tmp{}", std::process::id())); // várias sessões escrevem o mesmo arquivo
    let tmp = std::path::PathBuf::from(tmp);
    if std::fs::write(&tmp, input.as_bytes()).is_ok() {
        let _ = std::fs::rename(&tmp, usage_file);
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

/// Sessões abertas (uma por aba): pty e flag de parada do watcher de cada uma.
#[derive(Default)]
pub struct Sessions {
    tabs: Mutex<HashMap<String, (String, Arc<AtomicBool>)>>,
}

impl Sessions {
    /// Fecha a sessão da aba: para o watcher e mata o PTY (se ainda vivo). Idempotente.
    pub fn close(&self, tab_id: &str, pty: &PtyManager) {
        if let Some((pty_id, stop)) = self.tabs.lock().unwrap().remove(tab_id) {
            stop.store(true, Ordering::SeqCst);
            let _ = pty.kill(&pty_id);
        }
    }
}

/// O id da aba vira nome de pasta: só letras, dígitos, `-` e `_`.
pub fn valid_tab_id(id: &str) -> bool {
    !id.is_empty() && id.len() <= 64 && id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

/// Spawna o claude interativo da aba (task_id = `tab_id`), inicia o watcher dela e devolve o pty_id.
pub fn start(
    app: &AppHandle,
    session: &SessionConfig,
    tab_id: &str,
    model: Option<&str>,
    browser: bool,
    pty: &Arc<PtyManager>,
    sessions: &Sessions,
) -> Result<String, String> {
    if !valid_tab_id(tab_id) {
        return Err("id de aba inválido".into());
    }
    sessions.close(tab_id, pty); // recomeçar na mesma aba não pode duplicar PTY/watcher
    let odir = session.dir.join(".orchestra");
    let tdir = odir.join(tab_id);
    std::fs::create_dir_all(&tdir).map_err(|e| format!("falha ao criar .orchestra: {e}"))?;
    let events = tdir.join("events.jsonl");
    std::fs::write(&events, b"").map_err(|e| format!("falha ao criar events.jsonl: {e}"))?;
    exclude_orchestra(&session.dir);

    let exe = std::env::current_exe().map_err(|e| format!("current_exe: {e}"))?;
    let usage = odir.join("usage.json"); // uso é da conta: compartilhado entre as abas
    let refresh = crate::config::Config::load().usage_refresh_secs;
    let settings = hook_settings(&exe, &events, &usage, refresh);
    let argv = crate::tools::claude_argv(model, &settings, browser);

    // O observador de uso vive até o fim do app e é único.
    static USAGE: Once = Once::new();
    USAGE.call_once(|| {
        crate::usage::spawn_watcher(app.clone(), usage, std::time::Duration::from_secs(refresh));
    });
    let stop = crate::subagents::spawn_watcher(app.clone(), tab_id.to_string(), events);
    match pty.spawn(app, tab_id, argv, &session.dir, Box::new(|_| {})) {
        Ok(pty_id) => {
            sessions.tabs.lock().unwrap().insert(tab_id.to_string(), (pty_id.clone(), stop));
            Ok(pty_id)
        }
        Err(e) => {
            stop.store(true, Ordering::SeqCst);
            Err(e)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn tab_id_valido() {
        assert!(valid_tab_id("t1") && valid_tab_id("a-b_C9"));
        assert!(!valid_tab_id("") && !valid_tab_id("../x") && !valid_tab_id("a b"));
    }

    #[test]
    fn quoting() {
        assert_eq!(shell_quote("a b"), "'a b'");
        assert_eq!(shell_quote("it's"), r"'it'\''s'");
    }

    #[test]
    fn settings_json() {
        let s = hook_settings(
            Path::new("/opt/my app/orchestra"),
            Path::new("/p/e.jsonl"),
            Path::new("/p/u.json"),
            5,
        );
        let v: Value = serde_json::from_str(&s).unwrap();
        let cmd = "'/opt/my app/orchestra' hook '/p/e.jsonl'";
        assert_eq!(v["statusLine"]["type"], "command");
        assert_eq!(v["statusLine"]["command"], "'/opt/my app/orchestra' statusline '/p/u.json'");
        assert_eq!(v["statusLine"]["refreshInterval"], 5);
        assert_eq!(v["hooks"]["SubagentStart"][0]["hooks"][0]["command"], cmd);
        assert_eq!(v["hooks"]["SubagentStop"][0]["hooks"][0]["type"], "command");
        assert_eq!(v["hooks"]["PreToolUse"][0]["matcher"], "Agent|Task");
        assert_eq!(v["hooks"]["PreToolUse"][0]["hooks"][0]["command"], cmd);
        assert_eq!(v["hooks"]["PreToolUse"][1]["matcher"], "mcp__claude-in-chrome__.*");
        assert_eq!(v["hooks"]["PreToolUse"][1]["hooks"][0]["command"], cmd);
    }

    #[test]
    fn statusline_writes_atomically() {
        let f: PathBuf = std::env::temp_dir().join(format!("orch_usage_{}.json", std::process::id()));
        let _ = std::fs::remove_file(&f);
        run_statusline(&f, "lixo");
        assert!(!f.exists());
        run_statusline(&f, "{\"a\": 1}");
        assert_eq!(std::fs::read_to_string(&f).unwrap(), "{\"a\": 1}");
        let _ = std::fs::remove_file(&f);
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
