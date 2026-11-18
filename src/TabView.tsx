import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { ReactFlow, ReactFlowProvider, useReactFlow, type Edge, type Node } from "@xyflow/react";
import { api } from "./api";
import MainTerminal, { b64 } from "./MainTerminal";
import RootNode from "./flow/RootNode";
import SubagentNode, { type SubagentData } from "./flow/SubagentNode";
import { computeLayout } from "./flow/layout";
import SubagentModal from "./flow/SubagentModal";
import SetupScreen from "./SetupScreen";
import type { BrowserOption, ModelOption, SubagentLog } from "./types";

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

function Tree({ agents, onToggle, onOpen }: { agents: Agent[]; onToggle: (id: string) => void; onOpen: (id: string) => void }) {
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
        logs: a.logs, collapsed: collapsed[i], onToggle, onOpen, model: a.model,
      };
      ns.push({ id: a.id, type: "subagent", position: { x: b.x, y: b.y }, width: b.w, height: b.h, data, draggable: false });
      es.push({ id: `e-${a.id}`, source: "root", target: a.id, animated: a.status === "running", style: { stroke: a.status === "running" ? "#3b82f6" : "#374151" } });
    });
    return { nodes: ns, edges: es };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [agents, size, ckey, onToggle, onOpen]);

  useEffect(() => { rf.fitView({ padding: 0.02, duration: 150 }); }, [nodes.length, size, ckey, rf]);

  return (
    <div ref={box} style={{ width: "100%", height: "100%" }}>
      <ReactFlow nodes={nodes} edges={edges} nodeTypes={nodeTypes} fitView minZoom={0.2} maxZoom={1.5}
        nodesDraggable={false} nodesConnectable={false} proOptions={{ hideAttribution: true }}
        style={{ background: "#0b0d12" }} />
    </div>
  );
}

interface TabProps {
  tabId: string; active: boolean; canQuit: boolean; dir: string;
  options: ModelOption[]; browserOptions: BrowserOption[];
  onTitle: (title: string) => void; onReset: () => void; onBrowserOpen: () => void;
}

const baseName = (p: string) => p.split("/").filter(Boolean).pop() ?? p;

/** Uma sessão independente (aba): terminal do claude, árvore de subagentes e modal. */
export default function TabView({ tabId, active, canQuit, dir, options, browserOptions, onTitle, onReset, onBrowserOpen }: TabProps) {
  const [ptyId, setPtyId] = useState<string | null>(null);
  const [exitCode, setExitCode] = useState<number | null>(null);
  const [agents, setAgents] = useState<Agent[]>([]);
  const [split, setSplit] = useState(0.45);
  const [resizeKey, setResizeKey] = useState(0);
  const pending = useRef<Uint8Array[]>([]);
  const writer = useRef<((d: Uint8Array) => void) | null>(null);
  const idRef = useRef<string | null>(null);
  const early = useRef<{ pty_id: string; d: Uint8Array }[]>([]);
  const root = useRef<HTMLDivElement>(null);
  const models = useRef<Record<string, string>>({});
  const [chosen, setChosen] = useState<ModelOption | null>(null);
  const [starting, setStarting] = useState(false);
  const [startError, setStartError] = useState<string | null>(null);
  const [openId, setOpenId] = useState<string | null>(null);
  const aliveRef = useRef(true);
  const cb = useRef({ onTitle, onReset }); // callbacks do pai mudam a cada render: não podem re-registrar listeners
  cb.current = { onTitle, onReset };
  const deliver = useCallback((d: Uint8Array) => (writer.current ? writer.current(d) : pending.current.push(d)), []);
  const register = useCallback((w: ((d: Uint8Array) => void) | null) => { writer.current = w; }, []);

  // Sessão principal terminou bem (código 0): volta à tela inicial, limpando tudo da sessão anterior.
  const backToStart = useCallback(() => {
    idRef.current = null; early.current = []; pending.current.length = 0; models.current = {};
    setAgents([]); setOpenId(null); setExitCode(null); setPtyId(null); setChosen(null); setStartError(null);
    void api.closeTab(tabId).catch(() => {}); // para o watcher (e o PTY, se ainda vivo)
    cb.current.onTitle("Nova aba"); cb.current.onReset();
  }, [tabId]);
  const browserRef = useRef(onBrowserOpen);
  browserRef.current = onBrowserOpen;

  useEffect(() => {
    let alive = true; aliveRef.current = true;
    const un: (() => void)[] = [];
    const add = (p: Promise<() => void>) => p.then((u) => (alive ? un.push(u) : u()));
    (async () => {
      // Todos os listeners ANTES do startMain.
      await Promise.all([
        add(api.onPtyOutput((e) => {
          if (e.task_id !== tabId) return;
          const d = b64(e.data);
          if (idRef.current === null) early.current.push({ pty_id: e.pty_id, d });
          else if (e.pty_id === idRef.current) deliver(d);
        })),
        add(api.onPtyExit((e) => {
          if (e.task_id !== tabId || e.pty_id !== idRef.current) return;
          if (e.code === 0) backToStart(); else setExitCode(e.code);
        })),
        add(api.onSubagentStart((e) => e.tab_id === tabId && setAgents((s) => s.some((a) => a.id === e.agent_id) ? s : [...s, {
          id: e.agent_id, agentType: e.agent_type, description: e.description, status: "running",
          summary: null, logs: [], expanded: false, doneAt: null, model: models.current[e.agent_id] ?? null,
        }]))),
        add(api.onSubagentModel((e) => { if (e.tab_id !== tabId) return; models.current[e.agent_id] = e.model; setAgents((s) => s.map((a) => a.id === e.agent_id ? { ...a, model: e.model } : a)); })),
        add(api.onSubagentLog((e) => e.tab_id === tabId && setAgents((s) => s.map((a) => a.id === e.agent_id ? { ...a, logs: [...a.logs, e] } : a)))),
        add(api.onBrowserOpen((e) => { if (e.tab_id === tabId) browserRef.current(); })),
        add(api.onSubagentStop((e) => e.tab_id === tabId && setAgents((s) => s.map((a) => a.id === e.agent_id ? { ...a, status: "done", summary: e.last_message, doneAt: Date.now() } : a)))),
      ]);
    })();
    return () => { alive = false; aliveRef.current = false; un.forEach((u) => u()); };
  }, [deliver, backToStart, tabId]);

  // Ctrl+C na tela inicial fecha o app, mas só se for a única aba (no terminal o Ctrl+C é do Claude).
  useEffect(() => {
    if (chosen || !active || !canQuit) return;
    const onKey = (e: KeyboardEvent) => { if (e.ctrlKey && e.key.toLowerCase() === "c") { e.preventDefault(); void api.quitApp(); } };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [chosen, active, canQuit]);

  const start = useCallback(async (opt: ModelOption, browser: boolean) => {
    setStarting(true); setStartError(null);
    try {
      const id = await api.startMain(tabId, opt.id === "default" ? null : opt.id, browser);
      if (!aliveRef.current) return;
      idRef.current = id;
      early.current.filter((x) => x.pty_id === id).forEach((x) => deliver(x.d));
      early.current = [];
      setChosen(opt);
      cb.current.onTitle(`${baseName(dir)} · ${opt.label}`);
      setPtyId(id);
    } catch (e) { setStartError(String(e)); }
    finally { setStarting(false); }
  }, [deliver, tabId, dir]);

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

  const openAgent = useCallback((id: string) => setOpenId(id), []);
  const closeModal = useCallback(() => setOpenId(null), []);
  const modalAgent = openId ? agents.find((a) => a.id === openId) : undefined;
  useEffect(() => { if (openId && !modalAgent) setOpenId(null); }, [openId, modalAgent]);

  const drag = (ev: React.PointerEvent) => {
    ev.preventDefault();
    const rect = root.current!.getBoundingClientRect();
    const move = (m: PointerEvent) => setSplit(Math.min(0.75, Math.max(0.25, (m.clientY - rect.top) / rect.height)));
    const up = () => { window.removeEventListener("pointermove", move); window.removeEventListener("pointerup", up); setResizeKey((k) => k + 1); };
    window.addEventListener("pointermove", move);
    window.addEventListener("pointerup", up);
  };

  if (!chosen) {
    return <SetupScreen dir={dir} options={options} browserOptions={browserOptions} starting={starting} error={startError} onStart={start} />;
  }

  return (
    <div ref={root} style={{ height: "100%", display: "flex", flexDirection: "column", overflow: "hidden" }}>
      <div style={{ height: showTree ? `${split * 100}%` : "100%", transition: "height 0.25s ease", minHeight: 0, display: "flex", flexDirection: "column" }}>
        {exitCode !== null && (
          <div style={{ background: "#7f1d1d", color: "#fecaca", fontSize: 13, padding: "4px 12px", display: "flex", gap: 12, alignItems: "center" }}>
            sessão principal encerrada (código {exitCode})
            <button onClick={backToStart} style={{ background: "#991b1b", color: "#fff", border: 0, borderRadius: 4, padding: "2px 8px", cursor: "pointer" }}>Voltar ao início</button>
          </div>
        )}
        <div style={{ flex: 1, minHeight: 0 }}>
          <MainTerminal active={active} ptyId={ptyId} pending={pending.current} register={register} resizeKey={resizeKey} />
        </div>
      </div>
      {showTree && (<>
      <div onPointerDown={drag} style={{ height: 6, cursor: "row-resize", background: "#1f2430", flex: "none" }} />
      <div style={{ flex: 1, minHeight: 0 }}>
        <ReactFlowProvider><Tree agents={agents} onToggle={toggle} onOpen={openAgent} /></ReactFlowProvider>
      </div>
      </>)}
      {modalAgent && <SubagentModal agent={modalAgent} onClose={closeModal} />}
    </div>
  );
}
