// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]


fn main() {
    let session = orchestra_lib::cli::parse_and_wizard(std::env::args().skip(1).collect())
        .unwrap_or_else(|e| {
            eprintln!("{e}");
            std::process::exit(1);
        });
    orchestra_lib::run(session)
}
