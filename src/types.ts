// Contrato Rust <-> frontend. Espelha src-tauri/src/types.rs. NÃO alterar sem avisar o mantenedor.
export interface SessionConfig { dir: string; tool: string; provider: string; planner_model: string; goal: string; mock_planner: boolean }
export interface Task { id: string; title: string; prompt: string; depends_on: string[] }
export interface Plan { plan_summary: string; tasks: Task[] }
export type TaskStatus = "pending" | "running" | "merging" | "resolving" | "done" | "failed" | "needs_user";
export type ConflictAction = "retry" | "abort" | "manual_done";
export interface TaskStatusEvent { task_id: string; status: TaskStatus; pty_id: string | null }
export interface PtyOutput { pty_id: string; task_id: string; data: string /* base64 */ }
export interface PtyExit { pty_id: string; task_id: string; code: number }
