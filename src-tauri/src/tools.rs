//! Montagem do argv do Claude Code principal. Nunca usa shell: devolve argv puro.

/// Argv interativo (sem prompt: o usuário digita o objetivo no terminal).
pub fn claude_argv(model: Option<&str>, settings_json: &str) -> Vec<String> {
    let mut v: Vec<String> = ["claude", "--permission-mode", "acceptEdits", "--settings"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    v.push(settings_json.to_string());
    if let Some(m) = model {
        v.push("--model".into());
        v.push(m.to_string());
    }
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn argv_with_and_without_model() {
        let a = claude_argv(None, "{}");
        assert_eq!(a, ["claude", "--permission-mode", "acceptEdits", "--settings", "{}"]);
        let b = claude_argv(Some("opus"), "{}");
        assert_eq!(&b[5..], ["--model", "opus"]);
    }
}
