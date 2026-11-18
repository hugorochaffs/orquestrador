//! Adaptadores por ferramenta. Nunca usar shell: devolve argv puro.

/// Ids de ferramentas conhecidas.
pub const KNOWN_TOOLS: [&str; 3] = ["claude", "aider", "cline"];

/// Argv NÃO interativo/headless para executar `prompt` com a ferramenta (ex.: claude -p <prompt>).
/// `argv[0]` é o executável. Retorna None se a ferramenta é desconhecida.
pub fn build_argv(tool: &str, model: Option<&str>, prompt: &str) -> Option<Vec<String>> {
    let mut v: Vec<String> = match tool {
        "claude" => vec!["claude".into(), "-p".into(), prompt.into()],
        "aider" => vec!["aider".into(), "--message".into(), prompt.into(), "--yes-always".into()],
        "cline" => vec!["cline".into(), prompt.into()],
        _ => return None,
    };
    if let Some(m) = model.filter(|m| !m.is_empty()) {
        if tool != "cline" {
            v.push("--model".into());
            v.push(m.into());
        }
    }
    Some(v)
}

/// JSON de `--settings` com um hook Stop que dá `touch` no marker.
pub fn stop_hook_settings(marker: &std::path::Path) -> String {
    let quoted = format!("'{}'", marker.to_string_lossy().replace('\'', "'\\''"));
    serde_json::json!({"hooks": {"Stop": [{"hooks": [{"type": "command", "command": format!("touch {quoted}")}]}]}}).to_string()
}

/// Argv INTERATIVO (TUI). Para claude, aceita edições e sinaliza o fim do turno via `marker`.
pub fn build_interactive_argv(tool: &str, model: Option<&str>, prompt: &str, marker: &std::path::Path) -> Option<Vec<String>> {
    let model = model.filter(|m| !m.is_empty());
    match tool {
        "claude" => {
            let mut v: Vec<String> = ["claude", "--permission-mode", "acceptEdits", "--settings"].map(String::from).into();
            v.push(stop_hook_settings(marker));
            if let Some(m) = model {
                v.extend(["--model".into(), m.into()]);
            }
            v.push(prompt.into());
            Some(v)
        }
        "aider" => {
            let mut v: Vec<String> = vec!["aider".into(), "--message".into(), prompt.into()];
            if let Some(m) = model {
                v.extend(["--model".into(), m.into()]);
            }
            Some(v)
        }
        "cline" => Some(vec!["cline".into(), prompt.into()]),
        _ => None,
    }
}

pub fn known_tools() -> &'static [&'static str] {
    &KNOWN_TOOLS
}

/// Pares (provider, modelos sugeridos) para o wizard do planner.
pub fn suggested_models() -> Vec<(&'static str, Vec<&'static str>)> {
    vec![("anthropic", vec!["opus", "sonnet", "haiku"])]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn claude_with_model() {
        let a = build_argv("claude", Some("opus"), "oi").unwrap();
        assert_eq!(a, ["claude", "-p", "oi", "--model", "opus"]);
    }

    #[test]
    fn interactive_claude_hook() {
        let m = std::path::Path::new("/r/.orchestra/markers/t'1.done");
        let a = build_interactive_argv("claude", Some("opus"), "oi", m).unwrap();
        assert_eq!(&a[..4], ["claude", "--permission-mode", "acceptEdits", "--settings"]);
        let v: serde_json::Value = serde_json::from_str(&a[4]).unwrap();
        let cmd = v["hooks"]["Stop"][0]["hooks"][0]["command"].as_str().unwrap();
        assert_eq!(cmd, "touch '/r/.orchestra/markers/t'\\''1.done'");
        assert_eq!(&a[5..], ["--model", "opus", "oi"]);
        assert_eq!(build_interactive_argv("aider", None, "x", m).unwrap(), ["aider", "--message", "x"]);
        assert!(build_interactive_argv("foo", None, "x", m).is_none());
    }

    #[test]
    fn aider_and_unknown() {
        let a = build_argv("aider", None, "x").unwrap();
        assert_eq!(a, ["aider", "--message", "x", "--yes-always"]);
        assert!(build_argv("foo", None, "x").is_none());
    }
}
