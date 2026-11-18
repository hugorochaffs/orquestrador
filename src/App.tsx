import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { ReactFlow, ReactFlowProvider, useReactFlow, type Edge, type Node } from "@xyflow/react";
import { api } from "./api";
import MainTerminal, { b64 } from "./MainTerminal";
import RootNode from "./flow/RootNode";
import SubagentNode, { type SubagentData } from "./flow/SubagentNode";
import { computeLayout } from "./flow/layout";
import UsageBar from "./UsageBar";
import type { SubagentLog, UsageUpdate } from "./types";

interface Agent {
  id: string; agentType: string; description: string | null;
  status: "running" | "done"; summary: string | null; logs: SubagentLog[];
  expanded: boolean; // usuário reabriu um nó recolhido
  doneAt: number | null;
  model: string | null;
}
const nodeTypes = { root: RootNode, subagent: SubagentNode };
const COLLAPSE_MS = 3000;
const HIDE_MS = 4000;

function Tree({ agents, onToggle }: { agents: Agent[]; onToggle: (id: string) => void }) {
  const box = useRef<HTMLDivElement>(null);
  const [size, setSize] = useState({ w: 800, h: 400 });
  const [tick, setTick] = useState(0);
  const rf = useReactFlow();

  useEffect(() => {
    const ro = new ResizeObserver(([e]) => setSize({ w: e.contentRect.width, h: e.contentRect.height }));
    ro.observe(box.current!);
    return () => ro.disconnect();
  }, []);

  // Reavalia o recolhimento automático 3s após concluir.
  useEffect(() => {
    const pend = agents.filter((a) => a.doneAt && !a.expanded).map((a) => a.doneAt! + COLLAPSE_MS - Date.now()).filter((ms) => ms > 0);
    if (!pend.length) return;
    const t = setTimeout(() => setTick((n) => n + 1), Math.min(...pend) + 20);
    return () => clearTimeout(t);
  }, [agents, tick]);

  const collapsed = agents.map((a) => a.status === "done" && !a.expanded && Date.now() - (a.doneAt ?? 0) >= COLLAPSE_MS);
  const ckey = collapsed.join(",");
  const { nodes, edges } = useMemo(() => {
    const L = computeLayout(size.w, size.h, collapsed);
    const running = agents.filter((a) => a.status === "running").length;
    const ns: Node[] = [{
      id: "root", type: "root", position: { x: L.root.x, y: L.root.y }, width: L.root.w, height: L.root.h,
      data: { running, done: agents.length - running }, draggable: false,
    }];
    const es: Edge[] = [];
    agents.forEach((a, i) => {
      const b = L.children[i];
      const data: SubagentData = {
        agentType: a.agentType, description: a.description, status: a.status, summary: a.summary,
        logs: a.logs, collapsed: collapsed[i], onToggle, model: a.model,
      };
      ns.push({ id: a.id, type: "subagent", position: { x: b.x, y: b.y }, width: b.w, height: b.h, data, draggable: false });
      es.push({ id: `e-${a.id}`, source: "root", target: a.id, animated: a.status === "running", style: { stroke: a.status === "running" ? "#3b82f6" : "#374151" } });
    });
    return { nodes: ns, edges: es };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [agents, size, ckey, onToggle]);

  useEffect(() => { rf.fitView({ padding: 0.02, duration: 150 }); }, [nodes.length, size, ckey, rf]);

  return (
    <div ref={box} style={{ width: "100%", height: "100%" }}>
      <ReactFlow nodes={nodes} edges={edges} nodeTypes={nodeTypes} fitView minZoom={0.2} maxZoom={1.5}
        nodesDraggable={false} nodesConnectable={false} proOptions={{ hideAttribution: true }}
        style={{ background: "#0b0d12" }} />
    </div>
  );
}

export default function App() {
  const [ptyId, setPtyId] = useState<string | null>(null);
  const [exitCode, setExitCode] = useState<number | null>(null);
  const [agents, setAgents] = useState<Agent[]>([]);
  const [usage, setUsage] = useState<Record<string, UsageUpdate>>({});
  const [split, setSplit] = useState(0.45);
  const [resizeKey, setResizeKey] = useState(0);
  const pending = useRef<Uint8Array[]>([]);
  const writer = useRef<((d: Uint8Array) => void) | null>(null);
  const idRef = useRef<string | null>(null);
  const early = useRef<{ pty_id: string; d: Uint8Array }[]>([]);
  const root = useRef<HTMLDivElement>(null);
  const models = useRef<Record<string, string>>({});
  const register = useCallback((w: ((d: Uint8Array) => void) | null) => { writer.current = w; }, []);

  useEffect(() => {
    let alive = true;
    const un: (() => void)[] = [];
    const deliver = (d: Uint8Array) => (writer.current ? writer.current(d) : pending.current.push(d));
    const add = (p: Promise<() => void>) => p.then((u) => (alive ? un.push(u) : u()));
    (async () => {
      // Todos os listeners ANTES do startMain.
      await Promise.all([
        add(api.onPtyOutput((e) => {
          const d = b64(e.data);
          if (idRef.current === null) early.current.push({ pty_id: e.pty_id, d });
          else if (e.pty_id === idRef.current) deliver(d);
        })),
        add(api.onPtyExit((e) => { if (e.pty_id === idRef.current) setExitCode(e.code); })),
        add(api.onSubagentStart((e) => setAgents((s) => s.some((a) => a.id === e.agent_id) ? s : [...s, {
          id: e.agent_id, agentType: e.agent_type, description: e.description, status: "running",
          summary: null, logs: [], expanded: false, doneAt: null, model: models.current[e.agent_id] ?? null,
        }]))),
        add(api.onSubagentModel((e) => { models.current[e.agent_id] = e.model; setAgents((s) => s.map((a) => a.id === e.agent_id ? { ...a, model: e.model } : a)); })),
        add(api.onSubagentLog((e) => setAgents((s) => s.map((a) => a.id === e.agent_id ? { ...a, logs: [...a.logs, e] } : a)))),
        add(api.onUsageUpdate((e) => setUsage((s) => ({ ...s, [e.provider]: e })))),
        add(api.onSubagentStop((e) => setAgents((s) => s.map((a) => a.id === e.agent_id ? { ...a, status: "done", summary: e.last_message, doneAt: Date.now() } : a)))),
      ]);
      if (!alive) return;
      const id = await api.startMain();
      if (!alive) return;
      idRef.current = id;
      early.current.filter((x) => x.pty_id === id).forEach((x) => deliver(x.d));
      early.current = [];
      setPtyId(id);
    })();
    return () => { alive = false; un.forEach((u) => u()); };
  }, []);

  // Tudo concluído: espera 4s e some com a árvore (o terminal volta a 100%).
  const allDone = agents.length > 0 && agents.every((a) => a.status === "done");
  useEffect(() => {
    if (!allDone) return;
    const t = setTimeout(() => { models.current = {}; setAgents([]); }, HIDE_MS);
    return () => clearTimeout(t);
  }, [allDone, agents]);
  const showTree = agents.length > 0;
  useEffect(() => { // refaz o fit ao fim da transição de altura
    const t = setTimeout(() => setResizeKey((k) => k + 1), 300);
    return () => clearTimeout(t);
  }, [showTree]);

  const toggle = useCallback((id: string) => setAgents((s) => s.map((a) => a.id === id ? { ...a, expanded: !a.expanded } : a)), []);

  const drag = (ev: React.PointerEvent) => {
    ev.preventDefault();
    const rect = root.current!.getBoundingClientRect();
    const move = (m: PointerEvent) => setSplit(Math.min(0.75, Math.max(0.25, (m.clientY - rect.top) / rect.height)));
    const up = () => { window.removeEventListener("pointermove", move); window.removeEventListener("pointerup", up); setResizeKey((k) => k + 1); };
    window.addEventListener("pointermove", move);
    window.addEventListener("pointerup", up);
  };

  return (
    <div ref={root} style={{ height: "100vh", display: "flex", flexDirection: "column", overflow: "hidden" }}>
      <UsageBar usage={usage} />
      <div style={{ height: showTree ? `${split * 100}%` : "100%", transition: "height 0.25s ease", minHeight: 0, display: "flex", flexDirection: "column" }}>
        {exitCode !== null && (
          <div style={{ background: "#7f1d1d", color: "#fecaca", fontSize: 13, padding: "4px 12px" }}>sessão principal encerrada (código {exitCode})</div>
        )}
        <div style={{ flex: 1, minHeight: 0 }}>
          <MainTerminal ptyId={ptyId} pending={pending.current} register={register} resizeKey={resizeKey} />
        </div>
      </div>
      {showTree && (<>
      <div onPointerDown={drag} style={{ height: 6, cursor: "row-resize", background: "#1f2430", flex: "none" }} />
      <div style={{ flex: 1, minHeight: 0 }}>
        <ReactFlowProvider><Tree agents={agents} onToggle={toggle} /></ReactFlowProvider>
      </div>
      </>)}
    </div>
  );
}
