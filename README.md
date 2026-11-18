<h1 align="center">🎻 Orchestra</h1>

<p align="center">
  Uma interface local para o <b>Claude Code</b> com abas, orquestração de subagentes em paralelo<br>
  e uma árvore ao vivo mostrando o que cada subagente está fazendo.
</p>

---

## O que é

O Orchestra roda o `claude` de verdade (terminal real, com cores e interatividade) e acompanha tudo o que ele delega:

- **Orquestrador por padrão:** o primeiro chat é sempre um orquestrador. Ele divide a tarefa no maior número de subagentes possível, em paralelo, escolhe modelos baratos para tarefas simples, confere se todos retornaram certo e resolve conflitos.
- **Subagentes em modo auto**, e qualquer permissão adicional que precisem é pedida a você.
- **Subagente `infra` dedicado** para docker e shell de longa duração, para o ambiente não cair enquanto outros trabalham.
- **Árvore ao vivo:** cada subagente vira um nó com prompt, ferramentas e resultados em tempo real. Ao terminar, o nó recolhe num ✓; clique para reabrir o log.
- **Abas** no estilo do terminal do GNOME: cada aba é uma sessão independente (botão `+`, `×`, aba ativa sublinhada).
- **Navegador opcional por IA:** na tela inicial você escolhe se a IA pode usar o navegador. No Claude é o [Claude in Chrome](https://code.claude.com/docs/en/chrome) (extensão externa). Quando é usado, aparece uma aba `🌐 <nome da aba de origem>`.
- **Uso em tempo real** das janelas de 5h e semanal, atualizado a cada 5 s (configurável).

## Instalação

### Baixar pronto

Em [Releases](../../releases) estão os instaladores:

| Sistema | Arquivo |
| --- | --- |
| Debian/Ubuntu | `.deb` → `sudo apt install ./orchestra_*.deb` |
| Windows | `.msi` ou `.exe` |

Requisito: o [Claude Code](https://code.claude.com) instalado e logado (`npm install -g @anthropic-ai/claude-code`).

### Compilar do código-fonte

Pré-requisitos: [Rust](https://rustup.rs/) estável, [Node.js](https://nodejs.org/) 18+ e, no Linux, as dependências do [Tauri](https://v2.tauri.app/start/prerequisites/) (WebKitGTK 4.1 e GTK3).

```bash
git clone <url-do-repositório> orquestrador && cd orquestrador
npm install
npm run tauri build     # binário em src-tauri/target/release/orchestra
```

Para usar `orquestrador .` em qualquer lugar (Linux/Mac):

```bash
sudo ln -s "$PWD/src-tauri/target/release/orchestra" /usr/local/bin/orquestrador
```

## Como usar

```bash
cd ~/meu-projeto
orquestrador .
```

1. Na tela inicial escolha o provedor e o modelo (padrão `opusplan`) e, se quiser, marque **Permitir navegador**.
2. Digite seu objetivo no terminal principal. O orquestrador delega e a árvore aparece embaixo.
3. `+` abre uma nova aba (outra sessão); `×` fecha. Ctrl+C na tela inicial fecha o app.

A sessão roda em modo `auto` (Shift+Tab para alterar) e os diffs ficam no terminal, sem abrir IDE.

## Configuração

Arquivo `~/.config/orchestra/config.json` (criado na primeira execução):

```json
{
  "usage_refresh_secs": 5,
  "browser": { "claude": false }
}
```

| Chave | Descrição |
| --- | --- |
| `usage_refresh_secs` | Intervalo de atualização do uso, em segundos (mínimo 1). |
| `browser.<ia>` | Última escolha de "Permitir navegador" por IA. |

Mudanças valem ao iniciar uma sessão.

## Desenvolvimento

```bash
npm run tauri dev                          # app completo com hot reload
npm run dev                                # só a UI no navegador, com dados simulados
cd src-tauri && cargo test                 # testes do backend
npm run build                              # typecheck + build do frontend
```

## Releases

Ao criar uma tag `v*`, o workflow [`release.yml`](.github/workflows/release.yml) gera o `.deb` e os instaladores do Windows e os anexa a uma Release do GitHub:

```bash
git tag v0.1.0 && git push origin v0.1.0
```

## Arquitetura

- **Backend (Rust/Tauri, `src-tauri/src`):** `pty.rs` (terminal real via `portable-pty`), `session.rs` (sobe o `claude` com hooks), `subagents.rs` (lê eventos e transcripts), `usage.rs` (cota), `config.rs`, `tools.rs` (argv e prompt do orquestrador).
- **Frontend (React, `src`):** `xterm.js` para o terminal, `@xyflow/react` para a árvore, abas em `TabBar.tsx`/`TabView.tsx`.
- A árvore é alimentada por hooks do Claude Code (`SubagentStart`/`SubagentStop`/`PreToolUse`) gravados em `.orchestra/` no projeto (ignorado pelo git).
