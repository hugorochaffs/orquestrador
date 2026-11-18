// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::io::Read;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().map(String::as_str) == Some("hook") {
        // Subcomando chamado pelos hooks do Claude Code: nunca falha, nunca imprime.
        if let Some(file) = args.get(1) {
            let mut input = String::new();
            let _ = std::io::stdin().read_to_string(&mut input);
            orchestra_lib::session::run_hook(std::path::Path::new(file), &input);
        }
        return;
    }
    if args.first().map(String::as_str) == Some("statusline") {
        // Subcomando do statusLine do Claude Code: grava o JSON de uso; nunca imprime, sempre sai com 0.
        if let Some(file) = args.get(1) {
            let mut input = String::new();
            let _ = std::io::stdin().read_to_string(&mut input);
            orchestra_lib::session::run_statusline(std::path::Path::new(file), &input);
        }
        return;
    }
    let session = orchestra_lib::cli::parse_session(args).unwrap_or_else(|e| {
        eprintln!("{e}");
        std::process::exit(1);
    });
    orchestra_lib::run(session)
}
