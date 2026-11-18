import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { ReactFlow, Background, type ReactFlowInstance } from "@xyflow/react";
import "@xyflow/react/dist/style.css";
import { api } from "./api";
import TerminalNode from "./flow/TerminalNode";
import { layoutPlan, type NodeInfo } from "./flow/layout";
import { decode, pushOutput } from "./flow/store";
import type { Plan, TaskStatus } from "./types";

const nodeTypes = { terminal: TerminalNode };
const hhmmss = () => new Date().toLocaleTimeString("pt-BR");
const LABEL: Record<TaskStatus, string> = {
  pending: "pendente", running: "iniciou", merging: "entrou em merging", resolving: "resolvendo conflito",
  done: "concluída", failed: "falhou", needs_user: "precisa de você",
};

export default function App() {
  const [goal, setGoal] = useState("");
  const [plan, setPlan] = useState<Plan | null>(null);
  const [info, setInfo] = useState<Record<string, NodeInfo>>({});
  const [collapsed, setCollapsed] = useState<Set<string>>(new Set());
  const [log, setLog] = useState<{ t: string; m: string }[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [topPct, setTopPct] = useState(28);
  const [area, setArea] = useState({ w: 1200, h: 600 });
  const areaRef = useRef<HTMLDivElement>(null);
  const rootRef = useRef<HTMLDivElement>(null);
  const logRef = useRef<HTMLDivElement>(null);
  const rf = useRef<ReactFlowInstance | null>(null);
  const planRef = useRef<Plan | null>(null);
  const userOpen = useRef<Set<string>>(new Set());
  const addLog = useCallback((m: string) => setLog((l) => [...l, { t: hhmmss(), m }]), []);
  const title = (id: string) => planRef.current?.tasks.find((t) => t.id === id)?.title ?? id;

  const onToggle = useCallback((id: string) => {
    userOpen.current.add(id);
    setCollapsed((s) => { const n = new Set(s); if (n.has(id)) n.delete(id); else n.add(id); return n; });
  }, []);

  useEffect(() => {
    let alive = true;
    const un: Array<() => void> = [];
    (async () => {
      const subs = await Promise.all([
        api.onPlanReady((p) => {
          planRef.current = p; setPlan(p); setLoading(false);
          setInfo(Object.fromEntries(p.tasks.map((t) => [t.id, { status: "pending" as TaskStatus, ptyId: null }])));
          addLog(`Plano gerado: ${p.tasks.length} tarefas`);
        }),
        api.onTaskStatus((e) => {
          setInfo((m) => ({ ...m, [e.task_id]: { ...m[e.task_id], status: e.status, ptyId: e.pty_id ?? m[e.task_id]?.ptyId ?? null } }));
          addLog(`Tarefa "${title(e.task_id)}" ${LABEL[e.status]}`);
          if (e.status === "done") {
            setTimeout(() => {
              if (!userOpen.current.has(e.task_id)) setCollapsed((s) => new Set(s).add(e.task_id));
            }, 3000);
          } else {
            setCollapsed((s) => { if (!s.has(e.task_id)) return s; const n = new Set(s); n.delete(e.task_id); return n; });
          }
        }),
        api.onPtyOutput((e) => pushOutput(e.task_id, decode(e.data))),
        api.onPtyExit((e) => {
          addLog(`Terminal da tarefa "${title(e.task_id)}" encerrou (código ${e.code})`);
          if (e.code !== 0) setInfo((m) => m[e.task_id] ? { ...m, [e.task_id]: { ...m[e.task_id], reason: `Processo encerrou com código ${e.code}` } } : m);
        }),
      ]);
      if (!alive) { subs.forEach((u) => u()); return; }
      un.push(...subs);
      api.getSession().then((s) => setGoal(s.goal)).catch(() => {});
      api.startRun().catch((e) => { setError(String(e)); setLoading(false); addLog(`Erro do planner: ${e}`); });
    })();
    return () => { alive = false; un.forEach((u) => u()); };
  }, [addLog]);

  useEffect(() => {
    const el = areaRef.current; if (!el) return;
    const ro = new ResizeObserver(() => setArea({ w: el.clientWidth, h: el.clientHeight }));
    ro.observe(el);
    return () => ro.disconnect();
  }, []);

  useEffect(() => { logRef.current?.scrollTo(0, logRef.current.scrollHeight); }, [log]);

  const { nodes, edges } = useMemo(
    () => plan ? layoutPlan(plan, area, collapsed, info, onToggle) : { nodes: [], edges: [] },
    [plan, area, collapsed, info, onToggle],
  );
  const geom = nodes.map((n) => `${n.id}:${n.position.x | 0},${n.position.y | 0},${n.style?.width},${n.style?.height}`).join("|");
  useEffect(() => {
    const t = setTimeout(() => rf.current?.fitView({ padding: 0.02, duration: 200, minZoom: 0.1 }), 30);
    return () => clearTimeout(t);
  }, [geom]);

  const counts = useMemo(() => {
    const c: Record<string, number> = {};
    Object.values(info).forEach((i) => { c[i.status] = (c[i.status] ?? 0) + 1; });
    return c;
  }, [info]);
  const total = plan?.tasks.length ?? 0;

  const startDrag = (ev: React.MouseEvent) => {
    ev.preventDefault();
    const move = (e: MouseEvent) => {
      const r = rootRef.current!.getBoundingClientRect();
      setTopPct(Math.min(60, Math.max(12, ((e.clientY - r.top) / r.height) * 100)));
    };
    const up = () => { window.removeEventListener("mousemove", move); window.removeEventListener("mouseup", up); };
    window.addEventListener("mousemove", move); window.addEventListener("mouseup", up);
  };

  return (
    <div ref={rootRef} style={{ width: "100vw", height: "100vh", background: "#0b0d11", display: "flex", flexDirection: "column", color: "#e5e7eb", fontFamily: "system-ui" }}>
      <div style={{ height: `${topPct}%`, flex: "none", display: "flex", flexDirection: "column", padding: "8px 14px", gap: 4, minHeight: 0 }}>
        <div style={{ display: "flex", justifyContent: "space-between", gap: 12, fontSize: 14 }}>
          <b>Orquestrador{goal ? `: ${goal}` : ""}</b>
          <span style={{ whiteSpace: "nowrap" }}>
            {counts.done ?? 0}/{total} concluídas
            {(["running", "merging", "resolving", "needs_user", "failed", "pending"] as TaskStatus[]).filter((s) => counts[s]).map((s) => ` · ${counts[s]} ${s}`)}
          </span>
        </div>
        {plan && <div style={{ opacity: 0.75, fontSize: 13 }}>{plan.plan_summary}</div>}
        {loading && !error && <div style={{ color: "#60a5fa", fontSize: 13 }}>Planejando…</div>}
        {error && <div style={{ color: "#f87171", fontSize: 13 }}>Erro do planner: {error}</div>}
        <div ref={logRef} style={{ flex: 1, minHeight: 0, overflow: "auto", background: "#05070a", border: "1px solid #1f2937", borderRadius: 6, padding: "6px 8px", fontFamily: "ui-monospace, Menlo, Consolas, monospace", fontSize: 12 }}>
          {log.map((l, i) => <div key={i}><span style={{ opacity: 0.5 }}>{l.t}</span> {l.m}</div>)}
        </div>
      </div>
      <div onMouseDown={startDrag} style={{ height: 6, cursor: "row-resize", background: "#1f2937", flex: "none" }} />
      <div ref={areaRef} style={{ flex: 1, minHeight: 0 }}>
        <ReactFlow nodes={nodes} edges={edges} nodeTypes={nodeTypes} fitView minZoom={0.1} nodesDraggable={false}
          onInit={(i) => { rf.current = i; }} proOptions={{ hideAttribution: true }}>
          <Background />
        </ReactFlow>
      </div>
    </div>
  );
}
