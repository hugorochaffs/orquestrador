// Contrato Rust <-> frontend. Espelha src-tauri/src/types.rs. NÃO alterar sem avisar o mantenedor.
export interface SessionConfig { dir: string }
export interface ModelOption { provider: string; id: string; label: string; default: boolean }
export interface SubagentStart { tab_id: string; agent_id: string; agent_type: string; description: string | null; prompt: string | null }
export type LogKind = "user" | "text" | "tool" | "result";
export interface SubagentLog { tab_id: string; agent_id: string; kind: LogKind; text: string }
export interface SubagentModel { tab_id: string; agent_id: string; model: string }
export interface SubagentStop { tab_id: string; agent_id: string; last_message: string | null }
export interface PtyOutput { pty_id: string; task_id: string; data: string /* base64 */ }
export interface PtyExit { pty_id: string; task_id: string; code: number }
export interface UsageUpdate { provider: string; session_pct: number | null; session_resets_at: number | null; week_pct: number | null; week_resets_at: number | null }
export interface BrowserOpen { tab_id: string }
export interface BrowserOption { provider: string; label: string; note: string; enabled: boolean }
