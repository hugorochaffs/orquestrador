//! Montagem do argv do Claude Code principal. Nunca usa shell: devolve argv puro.

/// Instrução fixa anexada ao system prompt: o Claude sabe que roda dentro do Orchestra.
pub const ORCHESTRATOR_PROMPT: &str = "Você está rodando dentro do Orchestra, uma interface que mostra \
cada subagente seu como um nó numa árvore em tempo real. Atue como orquestrador: para qualquer tarefa que \
possa ser dividida, delegue as partes independentes a subagentes (ferramenta Agent), lançando vários em \
paralelo na mesma mensagem, e deixe-os fazer o trabalho pesado. Dê a cada subagente um prompt completo e \
autocontido, e depois integre e resuma os resultados para o usuário. Só trabalhe diretamente, sem subagentes, \
em tarefas triviais.";

/// Argv interativo (sem prompt: o usuário digita o objetivo no terminal).
pub fn claude_argv(model: Option<&str>, settings_json: &str) -> Vec<String> {
    let mut v: Vec<String> = ["claude", "--permission-mode", "acceptEdits", "--settings"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    v.push(settings_json.to_string());
    v.push("--append-system-prompt".into());
    v.push(ORCHESTRATOR_PROMPT.to_string());
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
        assert_eq!(&a[..5], ["claude", "--permission-mode", "acceptEdits", "--settings", "{}"]);
        assert_eq!(&a[5..], ["--append-system-prompt", ORCHESTRATOR_PROMPT]);
        let b = claude_argv(Some("opus"), "{}");
        assert_eq!(&b[7..], ["--model", "opus"]);
    }
}
