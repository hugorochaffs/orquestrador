//! Agendador do DAG + merge automático.
use crate::git::{self, MergeOutcome};
use crate::pty::PtyManager;
use crate::types::*;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter};
use tokio::sync::oneshot;
use tokio::task::JoinSet;

const RESOLVER_PROMPT: &str = "resolva os conflitos de merge neste repositório, remova os marcadores e finalize";

#[derive(Default)]
pub struct Orchestrator {
    pending: Mutex<HashMap<String, oneshot::Sender<ConflictAction>>>,
}

impl Orchestrator {
    /// Resposta do usuário a uma tarefa em `needs_user`.
    pub fn resolve(&self, task_id: &str, action: ConflictAction) -> Result<(), String> {
        let tx = self.pending.lock().unwrap().remove(task_id).ok_or_else(|| format!("a tarefa {task_id} não aguarda resolução"))?;
        tx.send(action).map_err(|_| "a execução já terminou".to_string())
    }
}

/// Executa uma tarefa (ou o resolvedor). Devolve o pty_id (se houver) e o canal do código de saída.
pub trait TaskRunner: Send + Sync + 'static {
    fn start(&self, task_id: &str, prompt: &str, cwd: &Path, resolver: bool) -> Result<(Option<String>, oneshot::Receiver<i32>), String>;
}

type Emit = Arc<dyn Fn(TaskStatusEvent) + Send + Sync>;

struct PtyRunner {
    app: AppHandle,
    pty: Arc<PtyManager>,
    tool: String,
}

impl TaskRunner for PtyRunner {
    fn start(&self, task_id: &str, prompt: &str, cwd: &Path, _resolver: bool) -> Result<(Option<String>, oneshot::Receiver<i32>), String> {
        let argv = crate::tools::build_argv(&self.tool, None, prompt).ok_or_else(|| format!("ferramenta desconhecida: {}", self.tool))?;
        let (tx, rx) = oneshot::channel();
        let id = self.pty.spawn(&self.app, task_id, argv, cwd, Box::new(move |c| {
            let _ = tx.send(c);
        }))?;
        Ok((Some(id), rx))
    }
}

/// Executa o plano inteiro; emite "task-status" (ver types.rs).
pub async fn run(o: Arc<Orchestrator>, app: AppHandle, s: SessionConfig, p: Plan, pty: Arc<PtyManager>) {
    let runner = Arc::new(PtyRunner { app: app.clone(), pty, tool: s.tool.clone() });
    let emit: Emit = Arc::new(move |e| {
        let _ = app.emit("task-status", e);
    });
    let run_id = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0).to_string();
    run_with(o, runner, emit, s.dir, run_id, p).await;
}

struct Ctx {
    o: Arc<Orchestrator>,
    runner: Arc<dyn TaskRunner>,
    emit: Emit,
    dir: PathBuf,
    run_id: String,
    merge_lock: tokio::sync::Mutex<()>,
}

impl Ctx {
    fn status(&self, id: &str, status: TaskStatus, pty_id: Option<String>) {
        (self.emit)(TaskStatusEvent { task_id: id.into(), status, pty_id });
    }
}

async fn run_with(o: Arc<Orchestrator>, runner: Arc<dyn TaskRunner>, emit: Emit, dir: PathBuf, run_id: String, plan: Plan) {
    let ctx = Arc::new(Ctx { o, runner, emit, dir: dir.clone(), run_id: run_id.clone(), merge_lock: Default::default() });
    for t in &plan.tasks {
        ctx.status(&t.id, TaskStatus::Pending, None);
    }
    let orig = match git::init_run(&dir, &run_id) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("orchestra: {e}");
            for t in &plan.tasks {
                ctx.status(&t.id, TaskStatus::Failed, None);
            }
            return;
        }
    };
    let (mut done, mut started, mut failed): (HashSet<String>, HashSet<String>, HashSet<String>) = Default::default();
    let mut set = JoinSet::new();
    loop {
        for t in &plan.tasks {
            if !started.contains(&t.id) && t.depends_on.iter().all(|d| done.contains(d)) {
                started.insert(t.id.clone());
                let (c, t) = (ctx.clone(), t.clone());
                set.spawn(async move {
                    let ok = exec_task(&c, &t).await;
                    (t.id, ok)
                });
            }
        }
        match set.join_next().await {
            None => break,
            Some(Ok((id, true))) => {
                done.insert(id);
            }
            Some(Ok((id, false))) => {
                failed.insert(id);
            }
            Some(Err(e)) => eprintln!("orchestra: tarefa abortou: {e}"),
        }
    }
    let ids: Vec<String> = plan.tasks.iter().map(|t| t.id.clone()).collect();
    let all_ok = done.len() == plan.tasks.len();
    let merged = all_ok && git::fast_forward(&dir, &orig, &run_id).map_err(|e| eprintln!("orchestra: {e}")).is_ok();
    git::cleanup(&dir, &run_id, &ids, merged);
}

async fn exec_task(c: &Ctx, t: &Task) -> bool {
    let wt = {
        let _g = c.merge_lock.lock().await; // a base precisa ser o estado atual da integração
        match git::add_task_worktree(&c.dir, &c.run_id, &t.id) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("orchestra: {e}");
                c.status(&t.id, TaskStatus::Failed, None);
                return false;
            }
        }
    };
    let (pty_id, rx) = match c.runner.start(&t.id, &t.prompt, &wt, false) {
        Ok(x) => x,
        Err(e) => {
            eprintln!("orchestra: {e}");
            c.status(&t.id, TaskStatus::Failed, None);
            return false;
        }
    };
    c.status(&t.id, TaskStatus::Running, pty_id.clone());
    if rx.await.unwrap_or(-1) != 0 {
        c.status(&t.id, TaskStatus::Failed, pty_id);
        return false;
    }
    c.status(&t.id, TaskStatus::Merging, pty_id.clone());
    let ok = merge_task(c, t, &wt, &pty_id).await;
    c.status(&t.id, if ok { TaskStatus::Done } else { TaskStatus::Failed }, pty_id);
    ok
}

async fn merge_task(c: &Ctx, t: &Task, wt: &Path, pty_id: &Option<String>) -> bool {
    let _g = c.merge_lock.lock().await;
    let ip = git::integration_path(&c.dir);
    let msg = format!("orchestra: {}", t.title);
    let outcome = git::commit_all(wt, &msg).and_then(|_| git::merge_branch(&ip, &git::task_branch(&c.run_id, &t.id), &format!("Merge da tarefa {}: {}", t.id, t.title)));
    match outcome {
        Ok(MergeOutcome::Clean) => true,
        Err(e) => {
            eprintln!("orchestra: {e}");
            false
        }
        Ok(MergeOutcome::Conflict(_)) => loop {
            c.status(&t.id, TaskStatus::Resolving, pty_id.clone());
            let rid = format!("{}-resolve", t.id);
            let mut pid = pty_id.clone();
            if let Ok((p, rx)) = c.runner.start(&rid, RESOLVER_PROMPT, &ip, true) {
                pid = p.or(pid);
                c.status(&t.id, TaskStatus::Resolving, pid.clone());
                if rx.await.unwrap_or(-1) == 0 && !git::has_conflict_markers(&ip) && git::commit_all(&ip, &format!("orchestra: resolve conflitos da tarefa {}", t.id)).is_ok() {
                    return true;
                }
            }
            let (tx, rx) = oneshot::channel();
            c.o.pending.lock().unwrap().insert(t.id.clone(), tx);
            c.status(&t.id, TaskStatus::NeedsUser, pid);
            match rx.await.unwrap_or(ConflictAction::Abort) {
                ConflictAction::Retry => continue,
                ConflictAction::Abort => {
                    let _ = git::abort_merge(&ip);
                    return false;
                }
                ConflictAction::ManualDone => {
                    if git::has_conflict_markers(&ip) || git::commit_all(&ip, &format!("orchestra: resolução manual da tarefa {}", t.id)).is_err() {
                        let _ = git::abort_merge(&ip);
                        return false;
                    }
                    return true;
                }
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tokio::sync::mpsc;

    /// Runner falso: executa a closure no cwd e já devolve o código.
    struct Fake(Box<dyn Fn(&str, &Path, bool) -> i32 + Send + Sync>, Mutex<Vec<String>>);
    impl TaskRunner for Fake {
        fn start(&self, id: &str, _p: &str, cwd: &Path, resolver: bool) -> Result<(Option<String>, oneshot::Receiver<i32>), String> {
            self.1.lock().unwrap().push(id.to_string());
            let (tx, rx) = oneshot::channel();
            let code = (self.0)(id, cwd, resolver);
            // atraso nas tarefas normais: garante paralelismo real (worktrees criados antes dos merges)
            tokio::spawn(async move {
                if !resolver {
                    tokio::time::sleep(std::time::Duration::from_millis(150)).await;
                }
                let _ = tx.send(code);
            });
            Ok((Some(format!("pty-{id}")), rx))
        }
    }

    fn task(id: &str, deps: &[&str]) -> Task {
        Task { id: id.into(), title: format!("tarefa {id}"), prompt: String::new(), depends_on: deps.iter().map(|s| s.to_string()).collect() }
    }

    fn start(name: &str, plan: Plan, f: impl Fn(&str, &Path, bool) -> i32 + Send + Sync + 'static) -> (PathBuf, Arc<Orchestrator>, Arc<Fake>, mpsc::UnboundedReceiver<TaskStatusEvent>, tokio::task::JoinHandle<()>) {
        let d = git::tests::temp_repo(name);
        let o = Arc::new(Orchestrator::default());
        let fake = Arc::new(Fake(Box::new(f), Default::default()));
        let (tx, rx) = mpsc::unbounded_channel();
        let emit: Emit = Arc::new(move |e| {
            let _ = tx.send(e);
        });
        let h = tokio::spawn(run_with(o.clone(), fake.clone(), emit, d.clone(), "rt".into(), plan));
        (d, o, fake, rx, h)
    }

    #[tokio::test]
    async fn ordem_do_dag_sem_conflito() {
        let plan = Plan { plan_summary: String::new(), tasks: vec![task("t1", &[]), task("t2", &["t1"])] };
        let (d, _o, fake, mut rx, h) = start("dag", plan, |id, cwd, r| {
            fs::write(cwd.join(format!("{id}.txt")), "x\n").unwrap();
            let _ = r;
            0
        });
        h.await.unwrap();
        assert_eq!(*fake.1.lock().unwrap(), vec!["t1", "t2"]);
        let mut done = vec![];
        while let Ok(e) = rx.try_recv() {
            if e.status == TaskStatus::Done {
                done.push(e.task_id);
            }
        }
        assert_eq!(done, vec!["t1", "t2"]);
        assert!(d.join("t1.txt").exists() && d.join("t2.txt").exists()); // fast-forward aplicado
        let _ = fs::remove_dir_all(&d);
    }

    #[tokio::test]
    async fn conflito_needs_user_e_manual_done() {
        let plan = Plan { plan_summary: String::new(), tasks: vec![task("t1", &[]), task("t2", &[]), task("t3", &["t1", "t2"])] };
        let (d, o, _f, mut rx, h) = start("conf", plan, |id, cwd, resolver| {
            if resolver {
                return 1; // resolvedor falha => needs_user
            }
            let c = if id == "t3" { "3\n".to_string() } else { format!("{id}\n") };
            fs::write(cwd.join(if id == "t3" { "c.txt" } else { "n.txt" }), c).unwrap();
            0
        });
        let mut t3_started = false;
        while let Some(e) = rx.recv().await {
            if e.status == TaskStatus::Running && e.task_id == "t3" {
                t3_started = true;
            }
            if e.status == TaskStatus::NeedsUser {
                assert!(!t3_started, "t3 não pode iniciar antes do conflito resolvido");
                fs::write(git::integration_path(&d).join("n.txt"), "resolvido\n").unwrap();
                o.resolve(&e.task_id, ConflictAction::ManualDone).unwrap();
            }
            if e.status == TaskStatus::Done && e.task_id == "t3" {
                break;
            }
        }
        h.await.unwrap();
        assert!(t3_started);
        assert_eq!(fs::read_to_string(d.join("n.txt")).unwrap(), "resolvido\n");
        assert!(d.join("c.txt").exists());
        let _ = fs::remove_dir_all(&d);
    }

    #[tokio::test]
    async fn falha_bloqueia_dependentes() {
        let plan = Plan { plan_summary: String::new(), tasks: vec![task("t1", &[]), task("t2", &["t1"])] };
        let (d, _o, fake, mut rx, h) = start("fail", plan, |_, _, _| 1);
        h.await.unwrap();
        assert_eq!(*fake.1.lock().unwrap(), vec!["t1"]);
        let mut st = vec![];
        while let Ok(e) = rx.try_recv() {
            st.push((e.task_id, e.status));
        }
        assert!(st.contains(&("t1".into(), TaskStatus::Failed)));
        assert!(!st.contains(&("t2".into(), TaskStatus::Running)));
        let _ = fs::remove_dir_all(&d);
    }
}
