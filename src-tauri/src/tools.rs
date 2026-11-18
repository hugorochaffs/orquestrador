//! Montagem do argv do Claude Code principal. Nunca usa shell: devolve argv puro.
use crate::types::{BrowserOption, ModelOption};

/// Instrução fixa anexada ao system prompt: o Claude sabe que roda dentro do Orchestra.
pub const ORCHESTRATOR_PROMPT: &str = "Você está rodando dentro do Orchestra, uma interface que mostra \
cada subagente seu como um nó numa árvore em tempo real. Você é SEMPRE o orquestrador deste projeto. \
Regras:\n\
1. Paralelismo máximo: divida toda tarefa não trivial no maior número possível de partes independentes e \
lance um subagente (ferramenta Agent) para cada uma, todos na MESMA mensagem, para terminar o mais rápido possível. \
Só trabalhe direto, sem subagentes, em tarefas triviais.\n\
2. Economia de tokens: você só planeja, delega e integra. Não leia nem edite arquivos grandes você mesmo; \
use subagentes para pesquisar, ler e editar. Escolha o modelo mais barato que dê conta de cada parte \
(haiku para buscas e tarefas simples, sonnet para código) e peça respostas curtas: só conclusões, caminhos de \
arquivos e o que mudou, sem despejar conteúdo.\n\
3. Todo subagente deve ser lançado com mode: \"auto\" (modo auto ligado). Dê a cada um um prompt completo e \
autocontido: objetivo, arquivos que ele pode tocar e formato do retorno.\n\
4. Sem conflitos: atribua a cada subagente um conjunto de arquivos disjunto; se duas partes precisarem do mesmo \
arquivo, serialize-as ou use isolation: \"worktree\". Se mesmo assim houver conflito (edições sobrepostas, \
merge, resultados contraditórios), você mesmo o resolve e confirma o resultado.\n\
5. Supervisão: acompanhe o retorno de cada subagente. Confira se todos terminaram e se o resultado está correto \
(rode build/testes quando existirem). Se algum falhar, travar ou voltar incompleto, relance-o com instruções \
corrigidas. Só dê a tarefa por concluída quando todos retornarem corretamente; então integre e resuma ao usuário.\n\
6. Permissões: se um subagente precisar de qualquer permissão adicional (ação bloqueada, acesso fora do escopo, \
comando ou ferramenta negada), ele deve parar e relatar o que precisa. Você então pede essa permissão ao usuário \
e só relança o subagente depois da aprovação. Nunca contorne, escale por conta própria nem presuma autorização.\n\
7. Infra: docker, serviços, servidores de desenvolvimento e comandos de shell de longa duração ficam com UM subagente \
dedicado, chamado \"infra\", lançado em segundo plano e mantido vivo. Os demais subagentes não sobem nem derrubam \
contêineres e serviços: pedem isso ao \"infra\" (SendMessage) ou a você, que repassa. Assim o ambiente nunca cai \
enquanto outro subagente executa sua tarefa. Pedidos que exijam permissão extra seguem a regra 6.";

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

/// Formas de navegador por IA. Hoje só o Claude (extensão Claude in Chrome, externa ao app).
/// `enabled` vem do config.json (chave "claude").
pub fn browser_options(cfg: &crate::config::Config) -> Vec<BrowserOption> {
    vec![BrowserOption {
        provider: "anthropic".into(),
        label: "Permitir navegador".into(),
        note: "Claude in Chrome: extensão do Chrome (externa); o Orchestra só marca a aba em uso".into(),
        enabled: cfg.browser_enabled("claude"),
    }]
}

/// Argv interativo (sem prompt: o usuário digita o objetivo no terminal).
/// Roda em modo `auto` e com `CLAUDE_CODE_AUTO_CONNECT_IDE=false` (só nesta sessão), para que os diffs
/// não abram no VS Code/JetBrains e fiquem no terminal. `browser` liga (`--chrome`) ou desliga (`--no-chrome`)
/// o Claude in Chrome.
pub fn claude_argv(model: Option<&str>, settings_json: &str, browser: bool) -> Vec<String> {
    let mut v: Vec<String> = ["env", "CLAUDE_CODE_AUTO_CONNECT_IDE=false", "claude", "--permission-mode", "auto", "--settings"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    v.push(settings_json.to_string());
    v.push(if browser { "--chrome" } else { "--no-chrome" }.into());
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
        let a = claude_argv(None, "{}", false);
        assert_eq!(
            &a[..8],
            ["env", "CLAUDE_CODE_AUTO_CONNECT_IDE=false", "claude", "--permission-mode", "auto", "--settings", "{}", "--no-chrome"]
        );
        assert_eq!(&a[8..], ["--append-system-prompt", ORCHESTRATOR_PROMPT]);
        let b = claude_argv(Some("opus"), "{}", false);
        assert_eq!(&b[10..], ["--model", "opus"]);
        assert_eq!(claude_argv(Some("default"), "{}", false).len(), a.len());
    }

    #[test]
    fn argv_chrome() {
        let on = claude_argv(None, "{}", true);
        assert!(on.contains(&"--chrome".to_string()) && !on.contains(&"--no-chrome".to_string()));
        let off = claude_argv(None, "{}", false);
        assert!(off.contains(&"--no-chrome".to_string()) && !off.contains(&"--chrome".to_string()));
        assert_eq!(on.len(), off.len());
    }

    #[test]
    fn browser_options_segue_config() {
        let c = crate::config::Config::parse("{\"browser\": {\"claude\": true}}");
        assert!(browser_options(&c)[0].enabled);
        assert_eq!(browser_options(&c)[0].provider, "anthropic");
        assert!(!browser_options(&crate::config::Config::default())[0].enabled);
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
