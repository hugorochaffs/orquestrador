//! Configuração do usuário em `$XDG_CONFIG_HOME/orchestra/config.json` (ou `~/.config/orchestra/config.json`).
//! Arquivo ausente ou inválido => valores padrão; chaves ausentes também.
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Intervalo, em segundos, de atualização do uso do Claude e das demais ferramentas.
    pub usage_refresh_secs: u64,
    /// Navegador permitido por IA (chave = nome da IA, ex.: "claude"). Ausente = desligado.
    pub browser: HashMap<String, bool>,
}

impl Default for Config {
    fn default() -> Self {
        Self { usage_refresh_secs: 5, browser: HashMap::new() }
    }
}

impl Config {
    pub fn parse(s: &str) -> Self {
        let mut c: Self = serde_json::from_str(s).unwrap_or_default();
        c.usage_refresh_secs = c.usage_refresh_secs.max(1);
        c
    }

    /// Se o navegador está permitido para a IA `key` (padrão: não).
    pub fn browser_enabled(&self, key: &str) -> bool {
        self.browser.get(key).copied().unwrap_or(false)
    }

    /// Grava a escolha de navegador de `key` no config.json, preservando as demais chaves. Nunca falha.
    pub fn save_browser(key: &str, enabled: bool) {
        let Some(path) = Self::path() else { return };
        let mut v: serde_json::Value = std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .filter(serde_json::Value::is_object)
            .unwrap_or_else(|| serde_json::json!({ "usage_refresh_secs": 5 }));
        v["browser"][key] = serde_json::Value::Bool(enabled);
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        if let Ok(s) = serde_json::to_string_pretty(&v) {
            let _ = std::fs::write(&path, s + "\n");
        }
    }

    pub fn path() -> Option<PathBuf> {
        let base = std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))?;
        Some(base.join("orchestra").join("config.json"))
    }

    /// Lê o arquivo de config; se não existir, grava um com os padrões para o usuário poder editar.
    pub fn load() -> Self {
        let Some(path) = Self::path() else { return Self::default() };
        match std::fs::read_to_string(&path) {
            Ok(s) => Self::parse(&s),
            Err(_) => {
                if let Some(dir) = path.parent() {
                    let _ = std::fs::create_dir_all(dir);
                }
                let _ = std::fs::write(&path, "{\n  \"usage_refresh_secs\": 5,\n  \"browser\": { \"claude\": false }\n}\n");
                Self::default()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn padrao_e_parse() {
        assert_eq!(Config::parse("lixo").usage_refresh_secs, 5);
        assert_eq!(Config::parse("{}").usage_refresh_secs, 5);
        assert_eq!(Config::parse("{\"usage_refresh_secs\": 12}").usage_refresh_secs, 12);
        assert_eq!(Config::parse("{\"usage_refresh_secs\": 0}").usage_refresh_secs, 1);
    }

    #[test]
    fn browser_por_ia() {
        assert!(!Config::parse("{}").browser_enabled("claude"));
        assert!(!Config::parse("lixo").browser_enabled("claude"));
        let c = Config::parse("{\"browser\": {\"claude\": true}, \"usage_refresh_secs\": 9}");
        assert!(c.browser_enabled("claude"));
        assert!(!c.browser_enabled("outra"));
        assert_eq!(c.usage_refresh_secs, 9);
        assert!(!Config::parse("{\"browser\": {\"claude\": false}}").browser_enabled("claude"));
        assert_eq!(Config::parse("{\"browser\": 3}"), Config::default()); // tipo inválido => padrões
    }
}
