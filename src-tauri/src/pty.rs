//! PtyManager com portable-pty.
use crate::types::{PtyExit, PtyOutput};
use base64::Engine;
use portable_pty::{native_pty_system, ChildKiller, CommandBuilder, MasterPty, PtySize};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter};

struct PtyEntry {
    master: Box<dyn MasterPty + Send>,
    writer: Box<dyn Write + Send>,
    killer: Box<dyn ChildKiller + Send + Sync>,
}

#[derive(Default)]
pub struct PtyManager {
    ptys: Arc<Mutex<HashMap<String, PtyEntry>>>,
    next: AtomicU64,
}

const NOT_FOUND: &str = "pty não encontrado";

impl PtyManager {
    /// Spawna `argv` (sem shell) em `cwd`. Emite "pty-output" e "pty-exit" (ver types.rs).
    /// `on_exit(code)` é chamado uma vez quando o processo termina. Retorna o pty_id.
    pub fn spawn(
        &self,
        app: &AppHandle,
        task_id: &str,
        argv: Vec<String>,
        cwd: &Path,
        on_exit: Box<dyn FnOnce(i32) + Send + 'static>,
    ) -> Result<String, String> {
        let (a1, a2) = (app.clone(), app.clone());
        let (t1, t2) = (task_id.to_string(), task_id.to_string());
        let id = format!("pty-{}", self.next.fetch_add(1, Ordering::SeqCst));
        let (i1, i2) = (id.clone(), id.clone());
        self.spawn_core(
            id,
            argv,
            cwd,
            Box::new(move |chunk| {
                let data = base64::engine::general_purpose::STANDARD.encode(chunk);
                let _ = a1.emit(
                    "pty-output",
                    PtyOutput { pty_id: i1.clone(), task_id: t1.clone(), data },
                );
            }),
            Box::new(move |code| {
                let _ = a2.emit("pty-exit", PtyExit { pty_id: i2, task_id: t2, code });
                on_exit(code);
            }),
        )
    }

    fn spawn_core(
        &self,
        id: String,
        argv: Vec<String>,
        cwd: &Path,
        on_data: Box<dyn Fn(&[u8]) + Send + 'static>,
        on_exit: Box<dyn FnOnce(i32) + Send + 'static>,
    ) -> Result<String, String> {
        if argv.is_empty() {
            return Err("argv vazio".into());
        }
        let pair = native_pty_system()
            .openpty(PtySize { rows: 24, cols: 80, pixel_width: 0, pixel_height: 0 })
            .map_err(|e| e.to_string())?;
        let mut cmd = CommandBuilder::new(&argv[0]);
        cmd.args(&argv[1..]);
        cmd.cwd(cwd);
        cmd.env("TERM", "xterm-256color");
        let mut child = pair.slave.spawn_command(cmd).map_err(|e| e.to_string())?;
        drop(pair.slave);
        let mut reader = pair.master.try_clone_reader().map_err(|e| e.to_string())?;
        let writer = pair.master.take_writer().map_err(|e| e.to_string())?;
        let entry = PtyEntry { master: pair.master, writer, killer: child.clone_killer() };
        self.ptys.lock().unwrap().insert(id.clone(), entry);

        let ptys = self.ptys.clone();
        let key = id.clone();
        std::thread::spawn(move || {
            let mut buf = [0u8; 8192];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => on_data(&buf[..n]),
                }
            }
            let code = child.wait().map(|s| s.exit_code() as i32).unwrap_or(-1);
            ptys.lock().unwrap().remove(&key);
            on_exit(code);
        });
        Ok(id)
    }

    pub fn write(&self, id: &str, data: &[u8]) -> Result<(), String> {
        let mut m = self.ptys.lock().unwrap();
        let e = m.get_mut(id).ok_or(NOT_FOUND)?;
        e.writer.write_all(data).and_then(|_| e.writer.flush()).map_err(|e| e.to_string())
    }

    pub fn resize(&self, id: &str, cols: u16, rows: u16) -> Result<(), String> {
        let m = self.ptys.lock().unwrap();
        let e = m.get(id).ok_or(NOT_FOUND)?;
        e.master
            .resize(PtySize { rows, cols, pixel_width: 0, pixel_height: 0 })
            .map_err(|e| e.to_string())
    }

    pub fn kill(&self, id: &str) -> Result<(), String> {
        let mut m = self.ptys.lock().unwrap();
        let e = m.get_mut(id).ok_or(NOT_FOUND)?;
        e.killer.kill().map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;
    use std::time::Duration;

    fn run(m: &PtyManager, id: &str, argv: &[&str]) -> (mpsc::Receiver<i32>, Arc<Mutex<Vec<u8>>>) {
        let out = Arc::new(Mutex::new(Vec::new()));
        let o = out.clone();
        let (tx, rx) = mpsc::channel();
        m.spawn_core(
            id.into(),
            argv.iter().map(|s| s.to_string()).collect(),
            Path::new("/tmp"),
            Box::new(move |c| o.lock().unwrap().extend_from_slice(c)),
            Box::new(move |code| {
                let _ = tx.send(code);
            }),
        )
        .unwrap();
        (rx, out)
    }

    fn text(out: &Arc<Mutex<Vec<u8>>>) -> String {
        String::from_utf8_lossy(&out.lock().unwrap()).to_string()
    }

    #[test]
    fn echo_captura_saida() {
        let m = PtyManager::default();
        let (rx, out) = run(&m, "a", &["echo", "ola"]);
        assert_eq!(rx.recv_timeout(Duration::from_secs(5)).unwrap(), 0);
        assert!(text(&out).contains("ola"));
    }

    #[test]
    fn codigo_de_saida() {
        let m = PtyManager::default();
        let (rx, _) = run(&m, "a", &["sh", "-c", "exit 3"]);
        assert_eq!(rx.recv_timeout(Duration::from_secs(5)).unwrap(), 3);
    }

    #[test]
    fn write_resize_kill() {
        let m = PtyManager::default();
        let (rx, out) = run(&m, "c", &["cat"]);
        m.resize("c", 100, 30).unwrap();
        m.write("c", b"xyz\n").unwrap();
        for _ in 0..50 {
            if text(&out).contains("xyz") {
                break;
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        assert!(text(&out).contains("xyz"));
        m.kill("c").unwrap();
        rx.recv_timeout(Duration::from_secs(5)).unwrap();
        assert!(m.kill("c").is_err());
    }
}
