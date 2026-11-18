// Fachada: usa o IPC do Tauri; no navegador (sem Tauri) usa um mock para validar a UI.
import * as ipc from "./ipc";
import type { LogKind, PtyExit, PtyOutput, SessionConfig, SubagentLog, SubagentStart, SubagentStop } from "./types";

type Unlisten = () => void;
export interface Api {
  getSession(): Promise<SessionConfig>;
  startMain(): Promise<string>;
  writePty(id: string, data: string): Promise<void>;
  resizePty(id: string, cols: number, rows: number): Promise<void>;
  onSubagentStart(cb: (e: SubagentStart) => void): Promise<Unlisten>;
  onSubagentLog(cb: (e: SubagentLog) => void): Promise<Unlisten>;
  onSubagentStop(cb: (e: SubagentStop) => void): Promise<Unlisten>;
  onPtyOutput(cb: (e: PtyOutput) => void): Promise<Unlisten>;
  onPtyExit(cb: (e: PtyExit) => void): Promise<Unlisten>;
}

const real: Api = { ...ipc };

function createMock(): Api {
  const subs: Record<string, Set<(e: never) => void>> = {};
  const on = <T,>(name: string) => (cb: (e: T) => void) => {
    (subs[name] ??= new Set()).add(cb as (e: never) => void);
    return Promise.resolve(() => void subs[name].delete(cb as (e: never) => void));
  };
  const emit = <T,>(name: string, e: T) => subs[name]?.forEach((cb) => (cb as (e: T) => void)(e));
  const PTY = "mock-pty";
  const out = (s: string) => emit<PtyOutput>("pty", { pty_id: PTY, task_id: "", data: btoa(s) });

  const agents = [
    { id: "a1", type: "Explore", desc: "Mapear a estrutura do projeto", at: 3000 },
    { id: "a2", type: "general-purpose", desc: "Implementar o componente de login", at: 5500 },
    { id: "a3", type: "code-reviewer", desc: "Revisar as mudanças", at: 9000 },
  ];
  const run = (a: (typeof agents)[number]) => {
    const log = (kind: LogKind, text: string) => emit<SubagentLog>("log", { agent_id: a.id, kind, text });
    emit<SubagentStart>("start", { agent_id: a.id, agent_type: a.type, description: a.desc, prompt: null });
    log("user", `Você é o subagente ${a.type}. Tarefa: ${a.desc}. Seja objetivo e reporte o resultado final.`);
    const steps: [LogKind, string][] = [
      ["text", "Vou começar lendo os arquivos relevantes."],
      ["tool", "Glob src/**/*.tsx"],
      ["result", Array.from({ length: 12 }, (_, i) => `src/arquivo${i}.tsx`).join("\n")],
      ["tool", "Read src/App.tsx"],
      ["result", "import React from 'react';\n// ... conteúdo longo do arquivo ...\nexport default function App() {}\n".repeat(4)],
      ["text", "Entendi a estrutura. Aplicando as alterações."],
      ["tool", "Edit src/App.tsx"],
      ["result", "Arquivo editado com sucesso."],
      ["tool", "Bash npm run build"],
      ["result", "build ok em 2.3s"],
      ["text", "Tudo pronto."],
    ];
    steps.forEach(([k, t], i) => setTimeout(() => log(k, t), 700 * (i + 1)));
    setTimeout(() => emit<SubagentStop>("stop", { agent_id: a.id, last_message: `${a.desc}: concluído sem erros.` }), 700 * (steps.length + 1));
  };

  return {
    getSession: async () => ({ dir: "/mock", model: null }),
    startMain: async () => {
      setTimeout(() => {
        out("\x1b[1;36mClaude Code (mock)\x1b[0m\r\n\x1b[32m>\x1b[0m Crie a tela de login\r\n");
        out("\x1b[33mVou delegar o trabalho a subagentes...\x1b[0m\r\n");
        agents.forEach((a) => setTimeout(() => { out(`\x1b[35m● Agent\x1b[0m(${a.desc})\r\n`); run(a); }, a.at));
      }, 300);
      return PTY;
    },
    writePty: async (_id, data) => out(data === "\r" ? "\r\n" : data),
    resizePty: async () => {},
    onSubagentStart: on<SubagentStart>("start"),
    onSubagentLog: on<SubagentLog>("log"),
    onSubagentStop: on<SubagentStop>("stop"),
    onPtyOutput: on<PtyOutput>("pty"),
    onPtyExit: on<PtyExit>("exit"),
  };
}

const isTauri = typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
export const api: Api = isTauri ? real : createMock();
