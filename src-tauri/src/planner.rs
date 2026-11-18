//! Planner (claude -p real + mock) e validação do DAG.
use crate::types::{Plan, SessionConfig, Task};
use std::collections::{HashMap, HashSet};

#[cfg(test)]
fn t(id: &str, deps: &[&str]) -> Task {
    Task { id: id.into(), title: String::new(), prompt: String::new(), depends_on: deps.iter().map(|s| s.to_string()).collect() }
}

fn task(id: &str, title: &str, prompt: &str, deps: &[&str]) -> Task {
    Task {
        id: id.into(),
        title: title.into(),
        prompt: prompt.into(),
        depends_on: deps.iter().map(|s| s.to_string()).collect(),
    }
}

fn mock_plan() -> Plan {
    Plan {
        plan_summary: "Plano de demonstração (mock)".into(),
        tasks: vec![
            task("t1", "Criar NOTES.md (parte A)", "Crie o arquivo NOTES.md com uma linha 'A'.", &[]),
            task("t2", "Criar NOTES.md (parte B)", "Crie o arquivo NOTES.md com uma linha 'B'.", &[]),
            task("t3", "Revisar NOTES.md", "Revise o NOTES.md e adicione uma linha final 'ok'.", &["t1", "t2"]),
        ],
    }
}

fn planner_prompt(goal: &str) -> String {
    format!(
        "Você é um planejador de engenharia de software. Analise o repositório do diretório atual e \
decomponha o objetivo abaixo em tarefas pequenas, o mais independentes possível, que possam rodar em \
paralelo por agentes de código; use depends_on apenas quando uma tarefa realmente precisa do resultado de outra. \
Responda SOMENTE com JSON válido, sem texto extra, no formato \
{{\"plan_summary\": string, \"tasks\": [{{\"id\": string, \"title\": string, \"prompt\": string, \"depends_on\": [string]}}]}}. \
Cada prompt deve ser autocontido.\n\nObjetivo: {goal}"
    )
}

/// Remove cercas ```json e isola o primeiro objeto JSON.
fn extract_json(s: &str) -> &str {
    let s = s.trim();
    match (s.find('{'), s.rfind('}')) {
        (Some(a), Some(b)) if b > a => &s[a..=b],
        _ => s,
    }
}

pub fn parse_plan(stdout: &str) -> Result<Plan, String> {
    let outer: serde_json::Value = serde_json::from_str(stdout.trim()).map_err(|e| format!("saída do planner inválida: {e}"))?;
    let text = match outer.get("result").and_then(|r| r.as_str()) {
        Some(r) => r.to_string(),
        None => outer.to_string(), // já é o próprio plano
    };
    serde_json::from_str(extract_json(&text)).map_err(|e| format!("plano inválido: {e}"))
}

pub async fn plan(s: &SessionConfig) -> Result<Plan, String> {
    let p = if s.mock_planner {
        mock_plan()
    } else {
        let mut cmd = tokio::process::Command::new("claude");
        cmd.arg("-p").arg(planner_prompt(&s.goal)).args(["--output-format", "json"]);
        if !s.planner_model.is_empty() {
            cmd.args(["--model", &s.planner_model]);
        }
        cmd.current_dir(&s.dir).stdin(std::process::Stdio::null());
        let out = cmd.output().await.map_err(|e| format!("falha ao executar claude: {e}"))?;
        if !out.status.success() {
            return Err(format!("planner falhou: {}", String::from_utf8_lossy(&out.stderr)));
        }
        parse_plan(&String::from_utf8_lossy(&out.stdout))?
    };
    validate(&p)?;
    Ok(p)
}

/// ids únicos, depends_on existentes, sem ciclos, ao menos 1 tarefa.
pub fn validate(p: &Plan) -> Result<(), String> {
    if p.tasks.is_empty() {
        return Err("o plano não tem tarefas".into());
    }
    let mut ids = HashSet::new();
    for t in &p.tasks {
        if t.id.trim().is_empty() {
            return Err("tarefa com id vazio".into());
        }
        if !ids.insert(t.id.as_str()) {
            return Err(format!("id duplicado: {}", t.id));
        }
    }
    let mut indeg: HashMap<&str, usize> = HashMap::new();
    for t in &p.tasks {
        for d in &t.depends_on {
            if !ids.contains(d.as_str()) {
                return Err(format!("tarefa {} depende de id inexistente: {d}", t.id));
            }
        }
        indeg.insert(&t.id, t.depends_on.len());
    }
    let mut queue: Vec<&str> = indeg.iter().filter(|(_, n)| **n == 0).map(|(k, _)| *k).collect();
    let mut seen = 0;
    while let Some(id) = queue.pop() {
        seen += 1;
        for t in p.tasks.iter().filter(|t| t.depends_on.iter().any(|d| d == id)) {
            let n = indeg.get_mut(t.id.as_str()).unwrap();
            *n -= 1;
            if *n == 0 {
                queue.push(&t.id);
            }
        }
    }
    if seen != p.tasks.len() {
        return Err("o plano tem dependência circular".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plan_of(ts: Vec<Task>) -> Plan {
        Plan { plan_summary: String::new(), tasks: ts }
    }

    #[test]
    fn mock_ok() {
        assert!(validate(&mock_plan()).is_ok());
    }
    #[test]
    fn ciclo() {
        let p = plan_of(vec![t("a", &["b"]), t("b", &["a"])]);
        assert!(validate(&p).unwrap_err().contains("circular"));
    }
    #[test]
    fn duplicado_vazio_inexistente() {
        assert!(validate(&plan_of(vec![t("a", &[]), t("a", &[])])).is_err());
        assert!(validate(&plan_of(vec![])).is_err());
        assert!(validate(&plan_of(vec![t("a", &["x"])])).is_err());
        assert!(validate(&plan_of(vec![t(" ", &[])])).is_err());
    }
    #[test]
    fn parse_com_cerca() {
        let inner = "```json\n{\"plan_summary\":\"s\",\"tasks\":[{\"id\":\"a\",\"title\":\"A\",\"prompt\":\"p\",\"depends_on\":[]}]}\n```";
        let outer = serde_json::json!({"type":"result","result":inner}).to_string();
        let p = parse_plan(&outer).unwrap();
        assert_eq!(p.tasks[0].id, "a");
    }
}
