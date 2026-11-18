// Fachada: IPC real no Tauri, mock no navegador.
import * as ipc from "./ipc";
import type { Plan, PtyExit, PtyOutput, SessionConfig, TaskStatusEvent } from "./types";

export const isMock = !(window as unknown as Record<string, unknown>).__TAURI_INTERNALS__;

type L<T> = (e: T) => void;
const ls = { plan: [] as L<Plan>[], st: [] as L<TaskStatusEvent>[], out: [] as L<PtyOutput>[], exit: [] as L<PtyExit>[] };
const reg = <T,>(arr: L<T>[]) => async (cb: L<T>) => { arr.push(cb); return () => { arr.splice(arr.indexOf(cb), 1); }; };

const plan: Plan = {
  plan_summary: "Plano mock: schema, API e UI em paralelo, depois integração.",
  tasks: [
    { id: "a", title: "Schema", prompt: "", depends_on: [] },
    { id: "b", title: "API", prompt: "", depends_on: ["a"] },
    { id: "c", title: "UI", prompt: "", depends_on: ["a"] },
    { id: "d", title: "Integração", prompt: "", depends_on: ["b", "c"] },
  ],
};

async function mockRun(): Promise<Plan> {
  await new Promise((r) => setTimeout(r, 800));
  ls.plan.forEach((f) => f(plan));
  const run = (id: string, delay: number, fail = false) => {
    setTimeout(() => {
      const pty = "pty-" + id;
      ls.st.forEach((f) => f({ task_id: id, status: "running", pty_id: pty }));
      let n = 0;
      const iv = setInterval(() => {
        ls.out.forEach((f) => f({ pty_id: pty, task_id: id, data: btoa(`\x1b[32m[${id}]\x1b[0m trabalhando ${++n}\r\n`) }));
        if (n >= 5) {
          clearInterval(iv);
          ls.st.forEach((f) => f({ task_id: id, status: fail ? "needs_user" : "done", pty_id: pty }));
        }
      }, 400);
    }, delay);
  };
  run("a", 500); run("b", 3500); run("c", 3500, true); run("d", 7000);
  return plan;
}

export const api = isMock
  ? {
      getSession: async (): Promise<SessionConfig> => ({ dir: "/mock", tool: "claude", provider: "x", planner_model: "m", goal: "Objetivo de exemplo (mock)", mock_planner: true }),
      startRun: mockRun,
      writePty: async (_i: string, _d: string) => {},
      resizePty: async (_i: string, _c: number, _r: number) => {},
      resolveConflict: async (t: string, a: string) => {
        ls.st.forEach((f) => f({ task_id: t, status: a === "abort" ? "failed" : "done", pty_id: "pty-" + t }));
      },
      onPlanReady: reg(ls.plan), onTaskStatus: reg(ls.st), onPtyOutput: reg(ls.out), onPtyExit: reg(ls.exit),
    }
  : {
      getSession: ipc.getSession, startRun: ipc.startRun, writePty: ipc.writePty, resizePty: ipc.resizePty,
      resolveConflict: ipc.resolveConflict as (t: string, a: string) => Promise<void>,
      onPlanReady: ipc.onPlanReady, onTaskStatus: ipc.onTaskStatus, onPtyOutput: ipc.onPtyOutput, onPtyExit: ipc.onPtyExit,
    };
