//! Worktrees, commit, merge, conflito (usa o binário `git` via std::process::Command).
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, PartialEq, Eq)]
pub enum MergeOutcome {
    Clean,
    Conflict(Vec<String>),
}

fn git_raw(cwd: &Path, args: &[&str]) -> Result<std::process::Output, String> {
    Command::new("git")
        .args(["-c", "user.name=Orchestra", "-c", "user.email=orchestra@localhost", "-c", "commit.gpgsign=false"])
        .args(args)
        .current_dir(cwd)
        .output()
        .map_err(|e| format!("falha ao executar git: {e}"))
}

pub fn git(cwd: &Path, args: &[&str]) -> Result<String, String> {
    let o = git_raw(cwd, args)?;
    if o.status.success() {
        Ok(String::from_utf8_lossy(&o.stdout).trim().to_string())
    } else {
        Err(format!("git {} falhou: {}", args.join(" "), String::from_utf8_lossy(&o.stderr).trim()))
    }
}

fn s(p: &Path) -> &str {
    p.to_str().unwrap_or_default()
}

pub fn integration_branch(run_id: &str) -> String {
    format!("orchestra/{run_id}")
}
pub fn task_branch(run_id: &str, task: &str) -> String {
    format!("orchestra/{run_id}-{task}")
}
pub fn integration_path(dir: &Path) -> PathBuf {
    dir.join(".orchestra").join("integration")
}
pub fn task_path(dir: &Path, task: &str) -> PathBuf {
    dir.join(".orchestra").join("worktrees").join(task)
}

/// Cria a branch e o worktree de integração; devolve o nome da branch original.
pub fn init_run(dir: &Path, run_id: &str) -> Result<String, String> {
    let orig = git(dir, &["rev-parse", "--abbrev-ref", "HEAD"])?;
    if orig == "HEAD" {
        return Err("HEAD destacado: faça checkout de uma branch".into());
    }
    // mantém .orchestra/ fora do git status
    let excl = PathBuf::from(git(dir, &["rev-parse", "--git-path", "info/exclude"])?);
    let excl = if excl.is_absolute() { excl } else { dir.join(excl) };
    if let Some(p) = excl.parent() {
        let _ = fs::create_dir_all(p);
    }
    let cur = fs::read_to_string(&excl).unwrap_or_default();
    if !cur.lines().any(|l| l.trim() == ".orchestra/") {
        let sep = if cur.is_empty() || cur.ends_with('\n') { "" } else { "\n" };
        fs::write(&excl, format!("{cur}{sep}.orchestra/\n")).map_err(|e| e.to_string())?;
    }
    let ip = integration_path(dir);
    fs::create_dir_all(ip.parent().unwrap()).map_err(|e| e.to_string())?;
    git(dir, &["worktree", "add", "-b", &integration_branch(run_id), s(&ip), "HEAD"])?;
    Ok(orig)
}

/// Worktree da tarefa, a partir do estado atual da integração.
pub fn add_task_worktree(dir: &Path, run_id: &str, task: &str) -> Result<PathBuf, String> {
    let p = task_path(dir, task);
    fs::create_dir_all(p.parent().unwrap()).map_err(|e| e.to_string())?;
    git(dir, &["worktree", "add", "-b", &task_branch(run_id, task), s(&p), &integration_branch(run_id)])?;
    Ok(p)
}

/// Faz commit de tudo (inclui concluir um merge em andamento); Ok(false) se não havia nada.
pub fn commit_all(wt: &Path, msg: &str) -> Result<bool, String> {
    git(wt, &["add", "-A"])?;
    let nothing = git_raw(wt, &["diff", "--cached", "--quiet"])?.status.success();
    let merging = git(wt, &["rev-parse", "-q", "--verify", "MERGE_HEAD"]).is_ok();
    if nothing && !merging {
        return Ok(false);
    }
    git(wt, &["commit", "--no-verify", "-m", msg])?;
    Ok(true)
}

pub fn merge_branch(integration: &Path, branch: &str, msg: &str) -> Result<MergeOutcome, String> {
    let o = git_raw(integration, &["merge", "--no-ff", "-m", msg, branch])?;
    if o.status.success() {
        return Ok(MergeOutcome::Clean);
    }
    let files = git(integration, &["diff", "--name-only", "--diff-filter=U"])?;
    if files.is_empty() {
        return Err(format!("merge falhou: {}", String::from_utf8_lossy(&o.stderr)));
    }
    Ok(MergeOutcome::Conflict(files.lines().map(String::from).collect()))
}

pub fn abort_merge(integration: &Path) -> Result<(), String> {
    git(integration, &["merge", "--abort"]).map(|_| ())
}

/// Marcadores de conflito no worktree.
pub fn has_conflict_markers(wt: &Path) -> bool {
    git_raw(wt, &["grep", "-q", "-E", "^(<<<<<<< |>>>>>>> |=======$)"]).map(|o| o.status.success()).unwrap_or(false)
}

/// Fast-forward da branch original para a integração, só se não divergiu.
pub fn fast_forward(dir: &Path, orig: &str, run_id: &str) -> Result<(), String> {
    let cur = git(dir, &["rev-parse", "--abbrev-ref", "HEAD"])?;
    if cur != orig {
        return Err(format!("a branch {orig} não está mais ativa (atual: {cur})"));
    }
    git(dir, &["merge", "--ff-only", &integration_branch(run_id)]).map(|_| ())
}

/// Remove worktrees e (se `delete_branches`) as branches da execução.
pub fn cleanup(dir: &Path, run_id: &str, tasks: &[String], delete_branches: bool) {
    for t in tasks {
        let _ = git(dir, &["worktree", "remove", "--force", s(&task_path(dir, t))]);
    }
    let _ = git(dir, &["worktree", "remove", "--force", s(&integration_path(dir))]);
    let _ = git(dir, &["worktree", "prune"]);
    if delete_branches {
        for t in tasks {
            let _ = git(dir, &["branch", "-D", &task_branch(run_id, t)]);
        }
        let _ = git(dir, &["branch", "-D", &integration_branch(run_id)]);
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    pub fn temp_repo(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("orchestra-test-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).unwrap();
        git(&d, &["init", "-q", "-b", "main"]).unwrap();
        fs::write(d.join("a.txt"), "base\n").unwrap();
        git(&d, &["add", "-A"]).unwrap();
        git(&d, &["commit", "-q", "-m", "inicial"]).unwrap();
        d
    }

    #[test]
    fn merge_limpo_e_ff() {
        let d = temp_repo("gitclean");
        let orig = init_run(&d, "r1").unwrap();
        assert_eq!(orig, "main");
        let w = add_task_worktree(&d, "r1", "t1").unwrap();
        fs::write(w.join("novo.txt"), "x\n").unwrap();
        assert!(commit_all(&w, "tarefa t1").unwrap());
        let ip = integration_path(&d);
        assert_eq!(merge_branch(&ip, &task_branch("r1", "t1"), "merge t1").unwrap(), MergeOutcome::Clean);
        assert!(git(&d, &["status", "--porcelain"]).unwrap().is_empty()); // .orchestra ignorado
        fast_forward(&d, "main", "r1").unwrap();
        assert!(d.join("novo.txt").exists());
        cleanup(&d, "r1", &["t1".into()], true);
        assert!(!ip.exists());
        let _ = fs::remove_dir_all(&d);
    }

    #[test]
    fn conflito_detectado() {
        let d = temp_repo("gitconf");
        init_run(&d, "r2").unwrap();
        let ip = integration_path(&d);
        let w1 = add_task_worktree(&d, "r2", "t1").unwrap();
        let w2 = add_task_worktree(&d, "r2", "t2").unwrap();
        fs::write(w1.join("a.txt"), "um\n").unwrap();
        fs::write(w2.join("a.txt"), "dois\n").unwrap();
        commit_all(&w1, "t1").unwrap();
        commit_all(&w2, "t2").unwrap();
        assert_eq!(merge_branch(&ip, &task_branch("r2", "t1"), "m1").unwrap(), MergeOutcome::Clean);
        match merge_branch(&ip, &task_branch("r2", "t2"), "m2").unwrap() {
            MergeOutcome::Conflict(f) => assert_eq!(f, vec!["a.txt"]),
            o => panic!("esperava conflito: {o:?}"),
        }
        assert!(has_conflict_markers(&ip));
        abort_merge(&ip).unwrap();
        assert!(!has_conflict_markers(&ip));
        let _ = fs::remove_dir_all(&d);
    }
}
