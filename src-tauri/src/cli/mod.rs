//! CLI: parse de args, validação do repo git e wizard dialoguer.
use crate::types::SessionConfig;
use dialoguer::{Confirm, Input, Select};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, PartialEq)]
pub struct Args {
    pub dir: PathBuf,
    pub mock_planner: bool,
}

#[derive(Debug, PartialEq)]
pub enum RepoState {
    Clean,
    Dirty,
    NotRepo,
    /// repo sem nenhum commit (HEAD não nascido)
    NoCommits,
}

pub fn parse_args(args: &[String]) -> Result<Args, String> {
    let mut dir = None;
    let mut mock = false;
    for a in args {
        if a == "--mock-planner" {
            mock = true;
        } else if a.starts_with("--") {
            return Err(format!("flag desconhecida: {a}"));
        } else if dir.is_none() {
            dir = Some(a.clone());
        } else {
            return Err(format!("argumento inesperado: {a}"));
        }
    }
    let dir = dir.ok_or("uso: orquestrador <dir> [--mock-planner]")?;
    let dir = std::fs::canonicalize(&dir).map_err(|e| format!("diretório '{dir}' inválido: {e}"))?;
    if !dir.is_dir() {
        return Err(format!("'{}' não é um diretório", dir.display()));
    }
    Ok(Args { dir, mock_planner: mock })
}

fn git(dir: &Path, args: &[&str]) -> Result<std::process::Output, String> {
    Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .map_err(|e| format!("falha ao executar git: {e}"))
}

pub fn repo_state(dir: &Path) -> Result<RepoState, String> {
    let r = git(dir, &["rev-parse", "--is-inside-work-tree"])?;
    if !r.status.success() {
        return Ok(RepoState::NotRepo);
    }
    if !git(dir, &["rev-parse", "--verify", "-q", "HEAD"])?.status.success() {
        return Ok(RepoState::NoCommits);
    }
    let s = git(dir, &["status", "--porcelain"])?;
    if !s.status.success() {
        return Err("git status falhou".into());
    }
    Ok(if s.stdout.is_empty() { RepoState::Clean } else { RepoState::Dirty })
}

fn run_git(dir: &Path, args: &[&str]) -> Result<(), String> {
    let o = git(dir, args)?;
    if o.status.success() {
        Ok(())
    } else {
        Err(format!("git falhou: {}", String::from_utf8_lossy(&o.stderr)))
    }
}

pub fn initial_commit(dir: &Path) -> Result<(), String> {
    run_git(
        dir,
        &["-c", "user.name=Orchestra", "-c", "user.email=orchestra@local",
          "commit", "--allow-empty", "-m", "commit inicial"],
    )
}

pub fn init_repo(dir: &Path) -> Result<(), String> {
    run_git(dir, &["init"])?;
    initial_commit(dir)
}

fn ui<T>(r: Result<T, dialoguer::Error>) -> Result<T, String> {
    r.map_err(|e| format!("wizard cancelado: {e}"))
}

pub fn parse_and_wizard(args: Vec<String>) -> Result<SessionConfig, String> {
    let a = parse_args(&args)?;
    match repo_state(&a.dir)? {
        RepoState::Clean => {}
        RepoState::Dirty => {
            return Err("working tree sujo: faça commit ou stash antes de iniciar".into());
        }
        RepoState::NotRepo => {
            let ok = ui(Confirm::new()
                .with_prompt("Não é um repositório git. Rodar `git init` + commit inicial vazio?")
                .default(true)
                .interact())?;
            if !ok {
                return Err("abortado: diretório não é repo git".into());
            }
            init_repo(&a.dir)?;
        }
        RepoState::NoCommits => {
            let ok = ui(Confirm::new()
                .with_prompt("O repositório não tem commits. Criar um commit inicial vazio?")
                .default(true)
                .interact())?;
            if !ok {
                return Err("abortado: o repositório precisa de ao menos um commit".into());
            }
            initial_commit(&a.dir)?;
            if repo_state(&a.dir)? == RepoState::Dirty {
                return Err("working tree sujo: faça commit ou stash antes de iniciar".into());
            }
        }
    }

    let tools = crate::detect::detect_tools();
    let tool = if tools.is_empty() {
        if !a.mock_planner {
            return Err("nenhuma ferramenta encontrada no PATH (claude, aider, cline)".into());
        }
        "claude".to_string()
    } else {
        let names: Vec<String> =
            tools.iter().map(|t| format!("{} ({})", t.id, t.path.display())).collect();
        let i = ui(Select::new().with_prompt("Ferramenta de execução").items(&names).default(0).interact())?;
        tools[i].id.clone()
    };

    let mut opts: Vec<(String, String)> = vec![];
    for (p, ms) in crate::tools::suggested_models() {
        for m in ms {
            opts.push((p.to_string(), m.to_string()));
        }
    }
    let labels: Vec<String> = opts.iter().map(|(p, m)| format!("{p} / {m}")).collect();
    let i = ui(Select::new().with_prompt("Provider / modelo do planner").items(&labels).default(0).interact())?;
    let (provider, planner_model) = opts[i].clone();

    let goal: String = ui(Input::new().with_prompt("Objetivo").interact_text())?;

    Ok(SessionConfig { dir: a.dir, tool, provider, planner_model, goal, mock_planner: a.mock_planner })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("orch_cli_{name}_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn parse_ok_and_errors() {
        let d = tmp("parse");
        let a = parse_args(&[d.to_string_lossy().into(), "--mock-planner".into()]).unwrap();
        assert!(a.mock_planner);
        assert!(parse_args(&[]).is_err());
        assert!(parse_args(&["/nao/existe/xyz".into()]).is_err());
        assert!(parse_args(&[d.to_string_lossy().into(), "--x".into()]).is_err());
    }

    #[test]
    fn repo_states() {
        let d = tmp("repo");
        assert_eq!(repo_state(&d).unwrap(), RepoState::NotRepo);
        run_git(&d, &["init"]).unwrap();
        assert_eq!(repo_state(&d).unwrap(), RepoState::NoCommits);
        initial_commit(&d).unwrap();
        assert_eq!(repo_state(&d).unwrap(), RepoState::Clean);
        std::fs::write(d.join("a.txt"), "x").unwrap();
        assert_eq!(repo_state(&d).unwrap(), RepoState::Dirty);
    }
}
