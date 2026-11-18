import { useEffect, useState } from "react";
import { ReactFlow, Background, type Edge, type Node } from "@xyflow/react";
import { api } from "./api";
import TerminalNode from "./flow/TerminalNode";
import { layoutPlan } from "./flow/layout";
import { decode, pushOutput } from "./flow/store";

const nodeTypes = { terminal: TerminalNode };

export default function App() {
  const [goal, setGoal] = useState("");
  const [summary, setSummary] = useState("");
  const [nodes, setNodes] = useState<Node[]>([]);
  const [edges, setEdges] = useState<Edge[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    let alive = true;
    const un: Array<() => void> = [];
    (async () => {
      const subs = await Promise.all([
        api.onPlanReady((p) => {
          const l = layoutPlan(p);
          setSummary(p.plan_summary); setNodes(l.nodes); setEdges(l.edges); setLoading(false);
        }),
        api.onTaskStatus((e) =>
          setNodes((ns) => ns.map((n) => n.id === e.task_id
            ? { ...n, data: { ...n.data, status: e.status, ptyId: e.pty_id ?? n.data.ptyId } } : n))),
        api.onPtyOutput((e) => pushOutput(e.task_id, decode(e.data))),
        api.onPtyExit(() => {}),
      ]);
      if (!alive) { subs.forEach((u) => u()); return; }
      un.push(...subs);
      api.getSession().then((s) => setGoal(s.goal)).catch(() => {});
      api.startRun().catch((e) => { setError(String(e)); setLoading(false); });
    })();
    return () => { alive = false; un.forEach((u) => u()); };
  }, []);

  return (
    <div style={{ width: "100vw", height: "100vh", background: "#0b0d11" }}>
      <div style={{ position: "absolute", zIndex: 10, top: 0, left: 0, right: 0, padding: "8px 14px", color: "#e5e7eb", background: "#111318cc", fontFamily: "system-ui", fontSize: 13 }}>
        <b>{goal}</b>
        {summary && <div style={{ opacity: 0.7 }}>{summary}</div>}
        {loading && !error && <div style={{ color: "#60a5fa" }}>Planejando tarefas…</div>}
        {error && <div style={{ color: "#f87171" }}>Erro do planner: {error}</div>}
      </div>
      <ReactFlow nodes={nodes} edges={edges} nodeTypes={nodeTypes} fitView minZoom={0.1}>
        <Background />
      </ReactFlow>
    </div>
  );
}
