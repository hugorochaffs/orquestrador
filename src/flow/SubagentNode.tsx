import { memo, useEffect, useRef, useState } from "react";
import { Handle, Position, type NodeProps } from "@xyflow/react";
import { CheckCircle2, ChevronDown, ChevronRight, FileText, Loader2, Pencil, Search, Terminal, Wrench } from "lucide-react";
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

function Entry({ e }: { e: SubagentLog }) {
  const [open, setOpen] = useState(false);
  if (e.kind === "text") return <div className="sa-text">{e.text}</div>;
  if (e.kind === "tool") return <div className="sa-tool"><ToolIcon text={e.text} /><span>{e.text}</span></div>;
  const long = e.text.length > 160 || e.text.includes("\n");
  const shown = open || !long ? e.text : e.text.split("\n")[0].slice(0, 160) + "…";
  return (
    <div className={e.kind === "user" ? "sa-user" : "sa-result"}>
      {e.kind === "user" && <span className="sa-lbl">prompt</span>}
      <pre>{shown}</pre>
      {long && <button onClick={() => setOpen(!open)}>{open ? "recolher" : "expandir"}</button>}
    </div>
  );
}

/** "claude-haiku-4-5-20251001" -> "haiku-4-5" */
export function shortModel(m: string) {
  return m.replace(/^claude-/, "").replace(/-\d{8}$/, "").replace(/\[.*\]$/, "");
}

function SubagentNode({ id, data }: NodeProps) {
  const d = data as SubagentData;
  const body = useRef<HTMLDivElement>(null);
  const done = d.status === "done";
  useEffect(() => {
    const el = body.current;
    if (el) el.scrollTop = el.scrollHeight;
  }, [d.logs.length, d.collapsed]);

  const title = `${d.agentType}${d.description ? " — " + d.description : ""}`;
  if (d.collapsed) {
    return (
      <div className="sa sa-compact" onClick={() => d.onToggle(id)} title={title}>
        <Handle type="target" position={Position.Top} />
        <CheckCircle2 size={16} color="#22c55e" />
        <span className="sa-ctitle">{title}</span>
        <ChevronRight size={14} />
      </div>
    );
  }
  return (
    <div className={`sa ${d.status}`}>
      <Handle type="target" position={Position.Top} />
      <div className="sa-head" onClick={done ? () => d.onToggle(id) : undefined}>
        <span className="sa-title" title={title}><b>{d.agentType}</b>{d.description ? ` — ${d.description}` : ""}</span>
        <span className="sa-model" title={d.model ?? "aguardando modelo"} style={{ fontSize: 10, padding: "1px 6px", borderRadius: 8, background: "#1f2430", color: "#9ca3af", opacity: d.model ? 1 : 0.5, whiteSpace: "nowrap", flex: "none" }}>{d.model ? shortModel(d.model) : "modelo…"}</span>
        <span className={`sa-badge ${d.status}`}>
          {done ? <CheckCircle2 size={12} /> : <Loader2 size={12} className="spin" />}
          {done ? "done" : "running"}
        </span>
        {done && <ChevronDown size={14} />}
      </div>
      <div className="sa-body nowheel nodrag" ref={body}>
        {d.logs.map((e, i) => <Entry key={i} e={e} />)}
      </div>
      {done && d.summary && <div className="sa-foot">{d.summary}</div>}
    </div>
  );
}
export default memo(SubagentNode);
