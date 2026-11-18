//! STUB (agente A): adaptadores por ferramenta. Nunca usar shell: devolve argv puro.
/// Argv NÃO interativo/headless para executar `prompt` com a ferramenta (ex.: claude -p <prompt>).
/// `argv[0]` é o executável. Retorna None se a ferramenta é desconhecida.
pub fn build_argv(_tool: &str, _model: Option<&str>, _prompt: &str) -> Option<Vec<String>> {
    None
}
