//! STUB (agente C): planner (claude -p real + mock) e validação do DAG.
use crate::types::{Plan, SessionConfig};

pub async fn plan(_s: &SessionConfig) -> Result<Plan, String> {
    Err("não implementado".into())
}
/// ids únicos, depends_on existentes, sem ciclos, ao menos 1 tarefa.
pub fn validate(_p: &Plan) -> Result<(), String> {
    Ok(())
}
