//! CLI: parse de args (`orquestrador [dir]`).
use crate::types::SessionConfig;
use std::path::PathBuf;

#[derive(Debug, PartialEq)]
pub struct Args {
    pub dir: PathBuf,
}

pub fn parse_args(args: &[String]) -> Result<Args, String> {
    let mut dir = None;
    for a in args {
        if a.starts_with("--") {
            return Err(format!("flag desconhecida: {a}"));
        } else if dir.is_none() {
            dir = Some(a.clone());
        } else {
            return Err(format!("argumento inesperado: {a}"));
        }
    }
    let dir = dir.unwrap_or_else(|| ".".to_string());
    let dir = std::fs::canonicalize(&dir).map_err(|e| format!("diretório '{dir}' inválido: {e}"))?;
    if !dir.is_dir() {
        return Err(format!("'{}' não é um diretório", dir.display()));
    }
    Ok(Args { dir })
}

/// Valida os args e o `claude` no PATH; a escolha do modelo é feita dentro do app.
pub fn parse_session(args: Vec<String>) -> Result<SessionConfig, String> {
    let a = parse_args(&args)?;
    crate::detect::find_claude()?;
    Ok(SessionConfig { dir: a.dir })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_ok_and_errors() {
        let d = std::env::temp_dir().join(format!("orch_cli_{}", std::process::id()));
        std::fs::create_dir_all(&d).unwrap();
        assert!(parse_args(&[d.to_string_lossy().into()]).is_ok());
        assert_eq!(parse_args(&[]).unwrap().dir, std::fs::canonicalize(".").unwrap());
        assert!(parse_args(&["/nao/existe/xyz".into()]).is_err());
        assert!(parse_args(&[d.to_string_lossy().into(), "--x".into()]).is_err());
        assert!(parse_args(&[d.to_string_lossy().into(), "--mock-planner".into()]).is_err());
    }
}
