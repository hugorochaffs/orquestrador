# 🎻 Orchestra

Orchestra is a local AI agent orchestrator. It bridges the gap between CLI-native AI coding tools (like Claude Code, Aider) and a visual, tree-based GUI.

Instead of opening multiple terminal tabs and losing track of context, Orchestra uses an LLM Planner to divide your prompt into sub-tasks (with dependencies), spawns real background terminal processes (PTYs) for each task, and visualizes them in a dynamic React Flow tree.

## ✨ Features
- **Auto-Detection:** Automatically finds installed AI CLIs in your `$PATH`.
- **Hybrid Interface:** Starts as a CLI prompt (`orquestrador .`), then boots a lightweight Tauri GUI.
- **True Terminal Emulation:** Uses `portable-pty` and `xterm.js` to run the actual CLI tools with full color and interactive support.
- **Isolated Parallel Work:** Each task runs in its own `git worktree`/branch; results are merged automatically into an integration branch. You only step in when the model cannot resolve a merge conflict.
- **Auto-Resolving Nodes:** Terminal nodes collapse to a ✓ upon successful execution (exit code 0); click to reopen the log.

## 🚀 Getting Started

### Prerequisites
- [Rust](https://rustup.rs/) (latest stable)
- [Node.js](https://nodejs.org/) (v18+)
- `git`
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
Navigate to any git project folder (clean working tree) and run:
```bash
orquestrador .
```
Follow the terminal prompts to select your tool, planner model and define your goal. The visual tree will handle the rest!

Use `orquestrador . --mock-planner` to try the UI with a fixed, fake plan (no LLM call).

## 🧪 Development
```bash
npm run tauri dev -- -- . --mock-planner   # run with hot reload
cd src-tauri && cargo test                 # backend tests
npm run build                              # typecheck + frontend build
```
