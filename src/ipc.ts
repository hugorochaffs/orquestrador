import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { BrowserOpen, BrowserOption, ModelOption, PtyExit, PtyOutput, SessionConfig, SubagentLog, SubagentModel, SubagentStart, SubagentStop, UsageUpdate } from "./types";

export const getSession = () => invoke<SessionConfig>("get_session");
/** Sobe o claude principal com o modelo escolhido na tela inicial (null = padrão do claude); devolve o pty_id. Registre os listeners ANTES. */
export const listModels = () => invoke<ModelOption[]>("list_models");
export const startMain = (tabId: string, model: string | null, browser: boolean) => invoke<string>("start_main", { tabId, model, browser });
export const closeTab = (tabId: string) => invoke<void>("close_tab", { tabId });
export const listBrowserOptions = () => invoke<BrowserOption[]>("list_browser_options");
export const onBrowserOpen = (cb: (e: BrowserOpen) => void): Promise<UnlistenFn> => listen<BrowserOpen>("browser-open", (e) => cb(e.payload));
export const writePty = (id: string, data: string) => invoke<void>("write_pty", { id, data });
export const resizePty = (id: string, cols: number, rows: number) => invoke<void>("resize_pty", { id, cols, rows });

export const onSubagentStart = (cb: (e: SubagentStart) => void): Promise<UnlistenFn> => listen<SubagentStart>("subagent-start", (e) => cb(e.payload));
export const onSubagentLog = (cb: (e: SubagentLog) => void): Promise<UnlistenFn> => listen<SubagentLog>("subagent-log", (e) => cb(e.payload));
export const onSubagentModel = (cb: (e: SubagentModel) => void): Promise<UnlistenFn> => listen<SubagentModel>("subagent-model", (e) => cb(e.payload));
export const onSubagentStop = (cb: (e: SubagentStop) => void): Promise<UnlistenFn> => listen<SubagentStop>("subagent-stop", (e) => cb(e.payload));
export const onPtyOutput = (cb: (e: PtyOutput) => void): Promise<UnlistenFn> => listen<PtyOutput>("pty-output", (e) => cb(e.payload));
export const onPtyExit = (cb: (e: PtyExit) => void): Promise<UnlistenFn> => listen<PtyExit>("pty-exit", (e) => cb(e.payload));
export const onUsageUpdate = (cb: (e: UsageUpdate) => void): Promise<UnlistenFn> => listen<UsageUpdate>("usage-update", (e) => cb(e.payload));
export const quitApp = () => invoke<void>("quit_app");
