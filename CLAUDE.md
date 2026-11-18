# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Regra inicial

NÃO COMMITAR DE FORMA ALGUMA COMO COAUTHOR. Nunca adicionar `Co-Authored-By` (nem qualquer linha de atribuição ao Claude) em mensagens de commit ou descrições de PR neste repositório.

## Visão geral

Orchestra: app Tauri 2 (Rust em `src-tauri/`, React em `src/`) que roda o `claude` num PTY com abas e mostra os subagentes numa árvore ao vivo. Detalhes e arquitetura no README.

## Comandos

- `npm run tauri dev`: app completo; `npm run dev`: só a UI com dados simulados
- `npm run build`: typecheck + build do frontend
- `cd src-tauri && cargo test`: testes do backend (um teste: `cargo test nome`)
- Release: tag `v*` dispara `.github/workflows/release.yml` (.deb e Windows)
