//! Observa o arquivo de uso gravado pelo subcomando `statusline` e emite "usage-update".
use crate::types::UsageUpdate;
use serde_json::Value;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};
use tauri::{AppHandle, Emitter};

fn pct(w: &Value) -> Option<f64> {
    w.get("used_percentage")?.as_f64()
}

fn resets(w: &Value) -> Option<i64> {
    let r = w.get("resets_at")?;
    r.as_i64().or_else(|| r.as_f64().map(|f| f as i64))
}

/// Converte o payload do statusLine em `UsageUpdate`. None se o JSON for inválido.
pub fn parse_usage(s: &str) -> Option<UsageUpdate> {
    let v: Value = serde_json::from_str(s).ok()?;
    let rl = v.get("rate_limits").unwrap_or(&Value::Null);
    let five = rl.get("five_hour").unwrap_or(&Value::Null);
    let seven = rl.get("seven_day").unwrap_or(&Value::Null);
    Some(UsageUpdate {
        provider: "claude".into(),
        session_pct: pct(five),
        session_resets_at: resets(five),
        week_pct: pct(seven),
        week_resets_at: resets(seven),
    })
}

/// `usage_file`: JSON único (sobrescrito a cada atualização) com o payload bruto do statusLine do Claude Code.
pub fn spawn_watcher(app: AppHandle, usage_file: PathBuf) {
    std::thread::spawn(move || {
        let mut last_mtime: Option<SystemTime> = None;
        let mut last: Option<UsageUpdate> = None;
        loop {
            std::thread::sleep(Duration::from_secs(1));
            let Ok(mtime) = std::fs::metadata(&usage_file).and_then(|m| m.modified()) else { continue };
            if last_mtime == Some(mtime) {
                continue;
            }
            let Ok(text) = std::fs::read_to_string(&usage_file) else { continue };
            last_mtime = Some(mtime);
            if let Some(u) = parse_usage(&text) {
                if last.as_ref() != Some(&u) {
                    let _ = app.emit("usage-update", &u);
                    last = Some(u);
                }
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn completo() {
        let u = parse_usage(r#"{"rate_limits":{"five_hour":{"used_percentage":23.5,"resets_at":1738425600},"seven_day":{"used_percentage":41.2,"resets_at":1738857600}}}"#).unwrap();
        assert_eq!(u.provider, "claude");
        assert_eq!(u.session_pct, Some(23.5));
        assert_eq!(u.session_resets_at, Some(1738425600));
        assert_eq!(u.week_pct, Some(41.2));
        assert_eq!(u.week_resets_at, Some(1738857600));
    }

    #[test]
    fn parcial() {
        let u = parse_usage(r#"{"rate_limits":{"five_hour":{"used_percentage":10,"resets_at":5}}}"#).unwrap();
        assert_eq!(u.session_pct, Some(10.0));
        assert_eq!(u.session_resets_at, Some(5));
        assert_eq!(u.week_pct, None);
        assert_eq!(u.week_resets_at, None);
    }

    #[test]
    fn ausente() {
        let u = parse_usage(r#"{"model":"x"}"#).unwrap();
        assert_eq!((u.session_pct, u.session_resets_at, u.week_pct, u.week_resets_at), (None, None, None, None));
    }

    #[test]
    fn lixo() {
        assert!(parse_usage("lixo").is_none());
        assert!(parse_usage("").is_none());
    }
}
