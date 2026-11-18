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
    fn aider_and_unknown() {
        let a = build_argv("aider", None, "x").unwrap();
        assert_eq!(a, ["aider", "--message", "x", "--yes-always"]);
        assert!(build_argv("foo", None, "x").is_none());
    }
}
