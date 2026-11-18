import { useEffect, useRef, useState } from "react";
import { Handle, Position, type NodeProps } from "@xyflow/react";
import { Terminal } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import { CheckCircle2 } from "lucide-react";
import { api } from "../api";
import { subscribe } from "./store";
import type { TaskStatus } from "../types";
import "./flow.css";

const COLORS: Record<TaskStatus, string> = {
  pending: "#6b7280", running: "#2563eb", merging: "#7c3aed", resolving: "#d97706",
  done: "#16a34a", failed: "#dc2626", needs_user: "#f59e0b",
};

export default function TerminalNode({ id, data }: NodeProps) {
  const { title, status, ptyId } = data as { title: string; status: TaskStatus; ptyId: string | null };
  const host = useRef<HTMLDivElement>(null);
  const term = useRef<Terminal | null>(null);
  const pty = useRef<string | null>(null);
  pty.current = ptyId;
  const [compact, setCompact] = useState(false);
  const [userToggled, setUserToggled] = useState(false);

  useEffect(() => {
    if (status !== "done" || userToggled) return;
    const t = setTimeout(() => setCompact(true), 3000);
    return () => clearTimeout(t);
  }, [status, userToggled]);

  useEffect(() => {
    const t = new Terminal({ fontSize: 12, convertEol: true, theme: { background: "#0b0d11" } });
    const f = new FitAddon();
    t.loadAddon(f);
    t.open(host.current!);
    term.current = t;
    const unsub = subscribe(id, (c) => t.write(c));
    const d = t.onData((s) => { if (pty.current) api.writePty(pty.current, s); });
    const ro = new ResizeObserver(() => {
      if (!host.current || host.current.clientWidth === 0) return;
      try { f.fit(); } catch { /* sem layout ainda */ }
      if (pty.current) api.resizePty(pty.current, t.cols, t.rows);
    });
    ro.observe(host.current!);
    return () => { ro.disconnect(); d.dispose(); unsub(); t.dispose(); };
  }, [id]);

  useEffect(() => {
    if (ptyId && term.current) api.resizePty(ptyId, term.current.cols, term.current.rows);
  }, [ptyId]);

  const toggle = () => { setUserToggled(true); setCompact((c) => !c); };

  return (
    <div className={`tnode ${status} ${compact ? "compact" : ""}`}>
      <Handle type="target" position={Position.Top} />
      <div className="head" onClick={status === "done" ? toggle : undefined}>
        <span style={{ display: "flex", gap: 6, alignItems: "center" }}>
          {status === "done" && <CheckCircle2 size={16} color="#22c55e" />}{title}
        </span>
        <span className="badge" style={{ background: COLORS[status] }}>{status}</span>
      </div>
      {status === "needs_user" && (
        <div className="actions">
          <button onClick={() => api.resolveConflict(id, "retry")}>Tentar de novo</button>
          <button onClick={() => api.resolveConflict(id, "abort")}>Abortar</button>
          <button onClick={() => api.resolveConflict(id, "manual_done")}>Já resolvi</button>
        </div>
      )}
      <div className="term nodrag nowheel" ref={host} />
      <Handle type="source" position={Position.Bottom} />
    </div>
  );
}
