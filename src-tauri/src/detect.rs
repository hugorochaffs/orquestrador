//! Detecção do `claude` no PATH.
use std::path::PathBuf;

pub fn find_claude() -> Result<PathBuf, String> {
    which::which("claude").map_err(|_| "`claude` não encontrado no PATH; instale o Claude Code".to_string())
}
