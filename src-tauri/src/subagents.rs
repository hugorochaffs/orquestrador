//! Observa o arquivo de eventos dos hooks e os transcripts dos subagentes.
use crate::types::{SubagentLog, SubagentStart, SubagentStop};
use serde_json::Value;
use std::collections::{HashMap, VecDeque};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tauri::{AppHandle, Emitter};

/// Evento produzido pelo núcleo.
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    Start(String, String, Option<String>, Option<String>),
    Log(String, String, String),
    Stop(String, Option<String>),
}

/// Leitor incremental de linhas completas de um arquivo.
#[derive(Default)]
struct Tail {
    offset: u64,
    partial: String,
}

impl Tail {
    fn read_lines(&mut self, path: &Path) -> Vec<String> {
        let Ok(mut f) = File::open(path) else { return vec![] };
        let len = f.metadata().map(|m| m.len()).unwrap_or(0);
        if len < self.offset {
            self.offset = 0;
            self.partial.clear();
        }
        if f.seek(SeekFrom::Start(self.offset)).is_err() {
            return vec![];
        }
        let mut buf = Vec::new();
        if f.read_to_end(&mut buf).is_err() {
            return vec![];
        }
        self.offset += buf.len() as u64;
        self.partial.push_str(&String::from_utf8_lossy(&buf));
        let mut out = Vec::new();
        while let Some(i) = self.partial.find('\n') {
            let line: String = self.partial.drain(..=i).collect();
            let line = line.trim();
            if !line.is_empty() {
                out.push(line.to_string());
            }
        }
        out
    }
}

struct Active {
    path: Option<PathBuf>,
    base: Option<PathBuf>, // diretório subagents
    tail: Tail,
}

struct Pending {
    description: Option<String>,
    prompt: Option<String>,
    subagent_type: Option<String>,
}

pub struct Watcher<F: FnMut(Event)> {
    events_file: PathBuf,
    tail: Tail,
    queue: VecDeque<Pending>,
    active: HashMap<String, Active>,
    emit: F,
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let t: String = s.chars().take(max).collect();
        format!("{t}…")
    }
}

fn str_of<'a>(v: &'a Value, k: &str) -> Option<&'a str> {
    v.get(k).and_then(|x| x.as_str())
}

fn summarize_input(name: &str, input: &Value) -> String {
    let pick = |keys: &[&str]| keys.iter().find_map(|k| input.get(*k).and_then(|x| x.as_str()));
    let body = match name {
        "Bash" => pick(&["command"]),
        "Read" | "Write" | "Edit" | "NotebookEdit" => pick(&["file_path", "notebook_path"]),
        _ => pick(&["command", "file_path", "path", "pattern", "query", "url", "description", "prompt"]),
    };
    let body = match body {
        Some(s) => s.to_string(),
        None => input.to_string(),
    };
    truncate(&format!("{name}: {body}"), 200)
}

fn result_text(c: &Value) -> String {
    match c {
        Value::String(s) => s.clone(),
        Value::Array(a) => a
            .iter()
            .filter_map(|b| b.get("text").and_then(|t| t.as_str()).map(String::from))
            .collect::<Vec<_>>()
            .join("\n"),
        _ => String::new(),
    }
}

/// Converte uma linha do transcript em pares (kind, texto).
pub fn parse_transcript_line(line: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let Ok(v) = serde_json::from_str::<Value>(line) else { return out };
    let ty = str_of(&v, "type").unwrap_or("");
    let Some(content) = v.get("message").and_then(|m| m.get("content")) else { return out };
    match content {
        Value::String(s) => {
            if ty == "user" && !s.trim().is_empty() {
                out.push(("user".into(), s.clone()));
            } else if ty == "assistant" && !s.trim().is_empty() {
                out.push(("text".into(), s.clone()));
            }
        }
        Value::Array(blocks) => {
            for b in blocks {
                match str_of(b, "type").unwrap_or("") {
                    "text" => {
                        let t = str_of(b, "text").unwrap_or("");
                        if !t.trim().is_empty() {
                            let kind = if ty == "user" { "user" } else { "text" };
                            out.push((kind.into(), t.to_string()));
                        }
                    }
                    "tool_use" => {
                        let name = str_of(b, "name").unwrap_or("?");
                        let input = b.get("input").cloned().unwrap_or(Value::Null);
                        out.push(("tool".into(), summarize_input(name, &input)));
                    }
                    "tool_result" => {
                        let t = result_text(b.get("content").unwrap_or(&Value::Null));
                        out.push(("result".into(), truncate(&t, 400)));
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    }
    out
}

fn find_transcript(dir: &Path, agent_id: &str) -> Option<PathBuf> {
    let direct = dir.join(format!("agent-{agent_id}.jsonl"));
    if direct.exists() {
        return Some(direct);
    }
    let plain = dir.join(format!("{agent_id}.jsonl"));
    if plain.exists() {
        return Some(plain);
    }
    let suffix = format!("{agent_id}.jsonl");
    std::fs::read_dir(dir)
        .ok()?
        .flatten()
        .map(|e| e.path())
        .find(|p| p.file_name().and_then(|n| n.to_str()).is_some_and(|n| n.ends_with(&suffix)))
}

impl<F: FnMut(Event)> Watcher<F> {
    pub fn new(events_file: PathBuf, emit: F) -> Self {
        Self { events_file, tail: Tail::default(), queue: VecDeque::new(), active: HashMap::new(), emit }
    }

    pub fn tick(&mut self) {
        for line in self.tail.read_lines(&self.events_file.clone()) {
            self.handle_hook(&line);
        }
        self.poll_transcripts();
    }

    fn handle_hook(&mut self, line: &str) {
        let Ok(v) = serde_json::from_str::<Value>(line) else { return };
        match str_of(&v, "hook_event_name").unwrap_or("") {
            "PreToolUse" => {
                if v.get("agent_id").is_some() {
                    return;
                }
                if matches!(str_of(&v, "tool_name"), Some("Agent") | Some("Task")) {
                    let ti = v.get("tool_input").cloned().unwrap_or(Value::Null);
                    self.queue.push_back(Pending {
                        description: str_of(&ti, "description").map(String::from),
                        prompt: str_of(&ti, "prompt").map(String::from),
                        subagent_type: str_of(&ti, "subagent_type").map(String::from),
                    });
                }
            }
            "SubagentStart" => {
                let id = str_of(&v, "agent_id").unwrap_or("").to_string();
                let ty = str_of(&v, "agent_type").unwrap_or("").to_string();
                if id.is_empty() {
                    return;
                }
                let idx = self
                    .queue
                    .iter()
                    .position(|p| p.subagent_type.as_deref() == Some(ty.as_str()))
                    .or(if self.queue.is_empty() { None } else { Some(0) });
                let p = idx.and_then(|i| self.queue.remove(i));
                let base = str_of(&v, "transcript_path").map(|t| {
                    PathBuf::from(t.strip_suffix(".jsonl").unwrap_or(t)).join("subagents")
                });
                self.active.insert(id.clone(), Active { path: None, base, tail: Tail::default() });
                (self.emit)(Event::Start(id, ty, p.as_ref().and_then(|p| p.description.clone()), p.and_then(|p| p.prompt)));
            }
            "SubagentStop" => {
                let id = str_of(&v, "agent_id").unwrap_or("").to_string();
                if let Some(mut a) = self.active.remove(&id) {
                    if let Some(p) = str_of(&v, "agent_transcript_path") {
                        a.path = Some(PathBuf::from(p));
                    }
                    self.drain(&id, &mut a);
                }
                (self.emit)(Event::Stop(id, str_of(&v, "last_assistant_message").map(String::from)));
            }
            _ => {}
        }
    }

    fn drain(&mut self, id: &str, a: &mut Active) {
        if a.path.is_none() {
            if let Some(b) = &a.base {
                a.path = find_transcript(b, id);
            }
        }
        let Some(p) = a.path.clone() else { return };
        for line in a.tail.read_lines(&p) {
            for (k, t) in parse_transcript_line(&line) {
                (self.emit)(Event::Log(id.to_string(), k, t));
            }
        }
    }

    fn poll_transcripts(&mut self) {
        let ids: Vec<String> = self.active.keys().cloned().collect();
        for id in ids {
            if let Some(mut a) = self.active.remove(&id) {
                self.drain(&id, &mut a);
                self.active.insert(id, a);
            }
        }
    }
}

/// `events_file`: JSONL em que cada linha é o JSON bruto de um hook do Claude Code.
/// Emite "subagent-start", "subagent-log", "subagent-stop".
pub fn spawn_watcher(app: AppHandle, events_file: PathBuf) {
    std::thread::spawn(move || {
        let mut w = Watcher::new(events_file, move |e| match e {
            Event::Start(agent_id, agent_type, description, prompt) => {
                let _ = app.emit("subagent-start", SubagentStart { agent_id, agent_type, description, prompt });
            }
            Event::Log(agent_id, kind, text) => {
                let _ = app.emit("subagent-log", SubagentLog { agent_id, kind, text });
            }
            Event::Stop(agent_id, last_message) => {
                let _ = app.emit("subagent-stop", SubagentStop { agent_id, last_message });
            }
        });
        loop {
            w.tick();
            std::thread::sleep(Duration::from_millis(300));
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn parse_lines() {
        let r = parse_transcript_line(r#"{"type":"user","message":{"role":"user","content":"oi"}}"#);
        assert_eq!(r, vec![("user".to_string(), "oi".to_string())]);
        let r = parse_transcript_line(
            r#"{"type":"assistant","message":{"content":[{"type":"text","text":"ola"},{"type":"tool_use","name":"Bash","input":{"command":"ls"}},{"type":"tool_use","name":"Read","input":{"file_path":"/a"}}]}}"#,
        );
        assert_eq!(r[0], ("text".into(), "ola".into()));
        assert_eq!(r[1], ("tool".into(), "Bash: ls".into()));
        assert_eq!(r[2], ("tool".into(), "Read: /a".into()));
        let r = parse_transcript_line(
            r#"{"type":"user","message":{"content":[{"type":"tool_result","content":[{"type":"text","text":"x"}]}]}}"#,
        );
        assert_eq!(r, vec![("result".to_string(), "x".to_string())]);
        assert!(parse_transcript_line("lixo").is_empty());
        assert!(parse_transcript_line(r#"{"type":"user"}"#).is_empty());
    }

    #[test]
    fn correlation_and_incremental() {
        let dir = std::env::temp_dir().join(format!("orch-sub-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("sess/subagents")).unwrap();
        let ev = dir.join("events.jsonl");
        let tr = dir.join("sess/subagents/agent-a1.jsonl");
        let mut events = Vec::new();
        {
            let mut w = Watcher::new(ev.clone(), |e| events.push(e));
            w.tick(); // arquivo inexistente
            let mut f = File::create(&ev).unwrap();
            writeln!(f, r#"{{"hook_event_name":"PreToolUse","tool_name":"Task","tool_input":{{"description":"d1","prompt":"p1","subagent_type":"Plan"}}}}"#).unwrap();
            writeln!(f, r#"{{"hook_event_name":"PreToolUse","tool_name":"Agent","agent_id":"x","tool_input":{{"description":"ignora"}}}}"#).unwrap();
            writeln!(f, r#"{{"hook_event_name":"PreToolUse","tool_name":"Agent","tool_input":{{"description":"d2","prompt":"p2","subagent_type":"Explore"}}}}"#).unwrap();
            writeln!(f, r#"{{"hook_event_name":"SubagentStart","agent_id":"a1","agent_type":"Explore","transcript_path":"{}/sess.jsonl"}}"#, dir.display()).unwrap();
            let mut t = File::create(&tr).unwrap();
            write!(t, r#"{{"type":"user","message":{{"content":"tarefa"}}}}"#).unwrap(); // parcial
            w.tick();
            writeln!(t).unwrap();
            w.tick();
            write!(f, r#"{{"hook_event_name":"SubagentStop","agent_id":"a1","last_assistant_message":"fim""#).unwrap(); // parcial
            w.tick();
            writeln!(f, "}}").unwrap();
            w.tick();
        }
        assert_eq!(events[0], Event::Start("a1".into(), "Explore".into(), Some("d2".into()), Some("p2".into())));
        assert_eq!(events[1], Event::Log("a1".into(), "user".into(), "tarefa".into()));
        assert_eq!(events[2], Event::Stop("a1".into(), Some("fim".into())));
        assert_eq!(events.len(), 3);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
