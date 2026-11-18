//! CLI: parse de args e wizard dialoguer.
use crate::types::SessionConfig;
use dialoguer::Select;
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
    let dir = dir.ok_or("uso: orchestra <dir>")?;
    let dir = std::fs::canonicalize(&dir).map_err(|e| format!("diretório '{dir}' inválido: {e}"))?;
    if !dir.is_dir() {
        return Err(format!("'{}' não é um diretório", dir.display()));
    }
    Ok(Args { dir })
}

pub fn parse_and_wizard(args: Vec<String>) -> Result<SessionConfig, String> {
    let a = parse_args(&args)?;
    crate::detect::find_claude()?;
    let models = [None, Some("opus"), Some("sonnet"), Some("haiku")];
    let labels = ["padrão", "opus", "sonnet", "haiku"];
    let i = Select::new()
        .with_prompt("Modelo")
        .items(&labels)
        .default(0)
        .interact()
        .map_err(|e| format!("wizard cancelado: {e}"))?;
    Ok(SessionConfig { dir: a.dir, model: models[i].map(String::from) })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_ok_and_errors() {
        let d = std::env::temp_dir().join(format!("orch_cli_{}", std::process::id()));
        std::fs::create_dir_all(&d).unwrap();
        assert!(parse_args(&[d.to_string_lossy().into()]).is_ok());
        assert!(parse_args(&[]).is_err());
        assert!(parse_args(&["/nao/existe/xyz".into()]).is_err());
        assert!(parse_args(&[d.to_string_lossy().into(), "--x".into()]).is_err());
        assert!(parse_args(&[d.to_string_lossy().into(), "--mock-planner".into()]).is_err());
    }
}
