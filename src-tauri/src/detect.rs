//! Detecção de ferramentas no PATH via `which`.
#[derive(Debug, Clone)]
pub struct DetectedTool {
    pub id: String,
    pub path: std::path::PathBuf,
}

pub fn detect_tools() -> Vec<DetectedTool> {
    crate::tools::known_tools()
        .iter()
        .filter_map(|id| which::which(id).ok().map(|path| DetectedTool { id: id.to_string(), path }))
        .collect()
}

#[cfg(test)]
mod tests {
    #[test]
    fn only_known_ids() {
        for t in super::detect_tools() {
            assert!(crate::tools::known_tools().contains(&t.id.as_str()));
        }
    }
}
