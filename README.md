# 🎻 Orchestra

Orchestra is a local GUI for Claude Code. A single real, interactive Claude Code session runs in the main terminal (top). Every subagent it spawns shows up as a live node in a tree (bottom), with its prompt, tool calls and results streaming in real time.

Permission prompts for subagents appear in the main terminal, exactly as in Claude Code.

## ✨ Features
- **Hybrid Interface:** starts as a CLI prompt (`orquestrador .`), then boots a lightweight Tauri GUI.
- **Real Terminal:** `portable-pty` + `xterm.js` run the actual `claude` TUI with full color and interactivity.
- **Live Subagent Tree:** built from Claude Code hooks (`SubagentStart`/`SubagentStop`/`PreToolUse`) and the subagent transcripts.
- **Auto-Collapsing Nodes:** finished subagents collapse to a ✓ card; click to reopen the log.

## 🚀 Getting Started

### Prerequisites
- [Rust](https://rustup.rs/) (latest stable)
- [Node.js](https://nodejs.org/) (v18+)
- Linux: WebKitGTK 4.1 and GTK3 dev packages (see the [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/))
- AI CLI tools installed globally (e.g., `npm install -g @anthropic-ai/claude-code`)

### Installation
1. Clone the repository.
2. Install frontend dependencies:
   ```bash
   npm install
   ```
3. Build the CLI/GUI combo:
   ```bash
   npm run tauri build
   ```

### Making it global
To use `orquestrador .` anywhere in your system:
- **Windows:** Add the `src-tauri/target/release` folder to your system's Environment Variables (PATH).
- **Mac/Linux:** Create a symlink:
  ```bash
  sudo ln -s /path/to/orchestra/src-tauri/target/release/orchestra /usr/local/bin/orquestrador
  ```

## 🎮 Usage
Navigate to any project folder and run:
```bash
orquestrador .
```
Pick the model in the terminal prompt, then type your goal in the main terminal. Ask Claude to use subagents (or let it decide) and the tree builds itself. The session starts in `acceptEdits` permission mode (Shift+Tab to change).

## 🧪 Development
```bash
npm run dev                                # UI in the browser with mock data
cd src-tauri && cargo test                 # backend tests
npm run build                              # typecheck + frontend build
```
