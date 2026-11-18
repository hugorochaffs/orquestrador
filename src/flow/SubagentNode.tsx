import { memo, useEffect, useRef, useState } from "react";
import { Handle, Position, type NodeProps } from "@xyflow/react";
import { CheckCircle2, ChevronDown, ChevronRight, FileText, Maximize2, Loader2, Pencil, Search, Terminal, Wrench } from "lucide-react";
import type { SubagentLog } from "../types";
import "./flow.css";

export interface SubagentData extends Record<string, unknown> {
  agentType: string;
  description: string | null;
  status: "running" | "done";
  summary: string | null;
  logs: SubagentLog[];
  model: string | null;
  collapsed: boolean;
  onToggle: (id: string) => void;
  onOpen: (id: string) => void;
}

function ToolIcon({ text }: { text: string }) {
  const t = text.split(/\s/)[0].toLowerCase();
  const p = { size: 13 };
  if (t === "bash") return <Terminal {...p} />;
  if (t === "read") return <FileText {...p} />;
  if (t === "edit" || t === "write") return <Pencil {...p} />;
  if (t === "grep" || t === "glob") return <Search {...p} />;
  return <Wrench {...p} />;
}

export function Entry({ e, big }: { e: SubagentLog; big?: boolean }) {
  const [open, setOpen] = useState(!!big && (e.kind === "user" || e.kind === "result"));
  if (e.kind === "text") return <div className="sa-text">{e.text}</div>;
  if (e.kind === "tool") return <div className="sa-tool"><ToolIcon text={e.text} /><span>{e.text}</span></div>;
  const long = e.text.length > 160 || e.text.includes("\n");
  const shown = open || !long ? e.text : e.text.split("\n")[0].slice(0, 160) + "…";
  return (
    <div className={e.kind === "user" ? "sa-user" : "sa-result"}>
      {e.kind === "user" && <span className="sa-lbl">prompt</span>}
      <pre>{shown}</pre>
      {long && <button onClick={(ev) => { ev.stopPropagation(); setOpen(!open); }}>{open ? "recolher" : "expandir"}</button>}
    </div>
  );
}

/** "claude-haiku-4-5-20251001" -> "haiku-4-5" */
export function shortModel(m: string) {
  return m.replace(/^claude-/, "").replace(/-\d{8}$/, "").replace(/\[.*\]$/, "");
}

export function LogList({ logs, big, running, className, onClick }: { logs: SubagentLog[]; big?: boolean; running: boolean; className: string; onClick?: () => void }) {
  const body = useRef<HTMLDivElement>(null);
  useEffect(() => {
    const el = body.current;
    if (el && (running || !big)) el.scrollTop = el.scrollHeight;
  }, [logs.length, running, big]);
  useEffect(() => { // modal abre no fim
    const el = body.current;
    if (el && big) el.scrollTop = el.scrollHeight;
  }, [big]);
  return <div className={className} ref={body} onClick={onClick}>{logs.map((e, i) => <Entry key={i} e={e} big={big} />)}</div>;
}

export function ModelBadge({ model }: { model: string | null }) {
  return <span className="sa-model" title={model ?? "aguardando modelo"} style={{ fontSize: 10, padding: "1px 6px", borderRadius: 8, background: "#1f2430", color: "#9ca3af", opacity: model ? 1 : 0.5, whiteSpace: "nowrap", flex: "none" }}>{model ? shortModel(model) : "modelo…"}</span>;
}

export function StatusBadge({ done }: { done: boolean }) {
  return (
    <span className={`sa-badge ${done ? "done" : "running"}`}>
      {done ? <CheckCircle2 size={12} /> : <Loader2 size={12} className="spin" />}
      {done ? "done" : "running"}
    </span>
  );
}

function SubagentNode({ id, data }: NodeProps) {
  const d = data as SubagentData;
  const done = d.status === "done";
  const title = `${d.agentType}${d.description ? " — " + d.description : ""}`;
  const expandBtn = (
    <button className="sa-icon nodrag nopan" title="Ampliar" onClick={(e) => { e.stopPropagation(); d.onOpen(id); }}><Maximize2 size={13} /></button>
  );
  if (d.collapsed) {
    return (
      <div className="sa sa-compact" onClick={() => d.onToggle(id)} title={title}>
        <Handle type="target" position={Position.Top} />
        <CheckCircle2 size={16} color="#22c55e" />
        <span className="sa-ctitle">{title}</span>
        {expandBtn}
        <ChevronRight size={14} />
      </div>
    );
  }
  const open = () => { if (!window.getSelection()?.toString()) d.onOpen(id); };
  return (
    <div className={`sa ${d.status}`}>
      <Handle type="target" position={Position.Top} />
      <div className="sa-head" style={{ cursor: "zoom-in" }} onClick={open}>
        <span className="sa-title" title={title}><b>{d.agentType}</b>{d.description ? ` — ${d.description}` : ""}</span>
        <ModelBadge model={d.model} />
        <StatusBadge done={done} />
        {expandBtn}
        {done && <button className="sa-icon nodrag nopan" title="Recolher" onClick={(e) => { e.stopPropagation(); d.onToggle(id); }}><ChevronDown size={14} /></button>}
      </div>
      <LogList logs={d.logs} running={!done} className="sa-body nowheel nodrag" onClick={open} />
      {done && d.summary && <div className="sa-foot">{d.summary}</div>}
    </div>
  );
}
export default memo(SubagentNode);
