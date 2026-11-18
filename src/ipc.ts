import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { ConflictAction, Plan, PtyExit, PtyOutput, SessionConfig, TaskStatusEvent } from "./types";

export const getSession = () => invoke<SessionConfig>("get_session");
export const startRun = () => invoke<Plan>("start_run");
export const writePty = (id: string, data: string) => invoke<void>("write_pty", { id, data });
export const resizePty = (id: string, cols: number, rows: number) => invoke<void>("resize_pty", { id, cols, rows });
export const resolveConflict = (taskId: string, action: ConflictAction) => invoke<void>("resolve_conflict", { taskId, action });

export const onPlanReady = (cb: (p: Plan) => void): Promise<UnlistenFn> => listen<{ plan: Plan }>("plan-ready", (e) => cb(e.payload.plan));
export const onTaskStatus = (cb: (e: TaskStatusEvent) => void): Promise<UnlistenFn> => listen<TaskStatusEvent>("task-status", (e) => cb(e.payload));
export const onPtyOutput = (cb: (e: PtyOutput) => void): Promise<UnlistenFn> => listen<PtyOutput>("pty-output", (e) => cb(e.payload));
export const onPtyExit = (cb: (e: PtyExit) => void): Promise<UnlistenFn> => listen<PtyExit>("pty-exit", (e) => cb(e.payload));
