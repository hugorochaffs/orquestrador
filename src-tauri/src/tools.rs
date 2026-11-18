//! Montagem do argv do Claude Code principal. Nunca usa shell: devolve argv puro.
use crate::types::ModelOption;

/// Instrução fixa anexada ao system prompt: o Claude sabe que roda dentro do Orchestra.
pub const ORCHESTRATOR_PROMPT: &str = "Você está rodando dentro do Orchestra, uma interface que mostra \
cada subagente seu como um nó numa árvore em tempo real. Atue como orquestrador: para qualquer tarefa que \
possa ser dividida, delegue as partes independentes a subagentes (ferramenta Agent), lançando vários em \
paralelo na mesma mensagem, e deixe-os fazer o trabalho pesado. Dê a cada subagente um prompt completo e \
autocontido, e depois integre e resuma os resultados para o usuário. Só trabalhe diretamente, sem subagentes, \
em tarefas triviais.";

/// Modelos que o Claude Code aceita em `--model`. O id "default" significa "sem --model".
pub fn models() -> Vec<ModelOption> {
    let items: [(&str, &str, bool); 11] = [
        ("opusplan", "Opus Plan — Opus planeja, Sonnet executa", true),
        ("opus", "Opus", false),
        ("sonnet", "Sonnet", false),
        ("haiku", "Haiku", false),
        ("sonnet[1m]", "Sonnet (contexto 1M)", false),
        ("opus[1m]", "Opus (contexto 1M)", false),
        ("default", "Padrão da conta", false),
        ("claude-fable-5-2", "Claude Fable 5.2", false),
        ("claude-opus-5-5", "Claude Opus 5.5", false),
        ("claude-sonnet-5-5", "Claude Sonnet 5.5", false),
        ("claude-haiku-5-5", "Claude Haiku 5.5", false),
    ];
    items
        .iter()
        .map(|(id, label, default)| ModelOption {
            provider: "anthropic".into(),
            id: (*id).into(),
            label: (*label).into(),
            default: *default,
        })
        .collect()
}

/// Argv interativo (sem prompt: o usuário digita o objetivo no terminal).
pub fn claude_argv(model: Option<&str>, settings_json: &str) -> Vec<String> {
    let mut v: Vec<String> = ["claude", "--permission-mode", "acceptEdits", "--settings"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    v.push(settings_json.to_string());
    v.push("--append-system-prompt".into());
    v.push(ORCHESTRATOR_PROMPT.to_string());
    if let Some(m) = model.filter(|m| *m != "default") {
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
        assert_eq!(claude_argv(Some("default"), "{}").len(), a.len());
    }

    #[test]
    fn models_one_default_unique_ids() {
        let m = models();
        assert_eq!(m.iter().filter(|x| x.default).count(), 1);
        assert_eq!(m.iter().find(|x| x.default).unwrap().id, "opusplan");
        let mut ids: Vec<_> = m.iter().map(|x| x.id.clone()).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), m.len());
    }
}
