import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { PtyExit, PtyOutput, SessionConfig, SubagentLog, SubagentModel, SubagentStart, SubagentStop, UsageUpdate } from "./types";

export const getSession = () => invoke<SessionConfig>("get_session");
/** Sobe o claude principal; devolve o pty_id. Registre os listeners ANTES. */
export const startMain = () => invoke<string>("start_main");
export const writePty = (id: string, data: string) => invoke<void>("write_pty", { id, data });
export const resizePty = (id: string, cols: number, rows: number) => invoke<void>("resize_pty", { id, cols, rows });

export const onSubagentStart = (cb: (e: SubagentStart) => void): Promise<UnlistenFn> => listen<SubagentStart>("subagent-start", (e) => cb(e.payload));
export const onSubagentLog = (cb: (e: SubagentLog) => void): Promise<UnlistenFn> => listen<SubagentLog>("subagent-log", (e) => cb(e.payload));
export const onSubagentModel = (cb: (e: SubagentModel) => void): Promise<UnlistenFn> => listen<SubagentModel>("subagent-model", (e) => cb(e.payload));
export const onSubagentStop = (cb: (e: SubagentStop) => void): Promise<UnlistenFn> => listen<SubagentStop>("subagent-stop", (e) => cb(e.payload));
export const onPtyOutput = (cb: (e: PtyOutput) => void): Promise<UnlistenFn> => listen<PtyOutput>("pty-output", (e) => cb(e.payload));
export const onPtyExit = (cb: (e: PtyExit) => void): Promise<UnlistenFn> => listen<PtyExit>("pty-exit", (e) => cb(e.payload));
export const onUsageUpdate = (cb: (e: UsageUpdate) => void): Promise<UnlistenFn> => listen<UsageUpdate>("usage-update", (e) => cb(e.payload));
