//! STUB (agente A): detecção de ferramentas no PATH via `which`.
#[derive(Debug, Clone)]
pub struct DetectedTool {
    pub id: String,
    pub path: std::path::PathBuf,
}

pub fn detect_tools() -> Vec<DetectedTool> {
    vec![]
}
