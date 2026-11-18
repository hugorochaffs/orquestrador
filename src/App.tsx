import { useCallback, useEffect, useRef, useState } from "react";
import { api } from "./api";
import TabView from "./TabView";
import TabBar, { type TabInfo } from "./TabBar";
import BrowserTab from "./BrowserTab";
import UsageBar from "./UsageBar";
import UpdateBar from "./UpdateBar";
import type { BrowserOption, ModelOption, UsageUpdate } from "./types";

interface Tab { id: string; title: string; kind: "session" | "browser"; originId?: string }

let seq = 0;
const newSession = (): Tab => ({ id: `t${++seq}`, title: "Nova aba", kind: "session" });

export default function App() {
  const [tabs, setTabs] = useState<Tab[]>(() => [newSession()]);
  const [activeId, setActiveId] = useState(() => tabs[0].id);
  const [usage, setUsage] = useState<Record<string, UsageUpdate>>({});
  const [dir, setDir] = useState("");
  const [options, setOptions] = useState<ModelOption[]>([]);
  const [browserOptions, setBrowserOptions] = useState<BrowserOption[]>([]);
  const tabsRef = useRef(tabs);
  tabsRef.current = tabs;

  useEffect(() => {
    let alive = true;
    let un: (() => void) | null = null;
    api.onUsageUpdate((e) => setUsage((s) => ({ ...s, [e.provider]: e }))).then((u) => (alive ? (un = u) : u()));
    Promise.all([api.getSession(), api.listModels(), api.listBrowserOptions()])
      .then(([s, m, b]) => { if (alive) { setDir(s.dir); setOptions(m); setBrowserOptions(b); } })
      .catch(() => {});
    return () => { alive = false; un?.(); };
  }, []);

  const setTitle = useCallback((id: string, title: string) => setTabs((t) => t.map((x) => x.id === id ? { ...x, title } : x)), []);
  // A sessão voltou à tela inicial: o navegador deixa de estar em uso por ela, então o marcador some.
  const dropMarker = useCallback((originId: string) => {
    setTabs((t) => t.filter((x) => x.originId !== originId));
    setActiveId((a) => tabsRef.current.find((x) => x.id === a)?.originId === originId ? originId : a);
  }, []);
  // Cada aba de origem tem no máximo uma aba marcador.
  const openMarker = useCallback((originId: string) => {
    setTabs((t) => {
      if (t.some((x) => x.originId === originId)) return t;
      const i = t.findIndex((x) => x.id === originId);
      if (i < 0) return t;
      const m: Tab = { id: `b${++seq}`, title: "", kind: "browser", originId };
      return [...t.slice(0, i + 1), m, ...t.slice(i + 1)];
    });
  }, []);

  const addTab = useCallback(() => {
    const t = newSession();
    setTabs((s) => [...s, t]);
    setActiveId(t.id);
  }, []);

  const closeTab = useCallback((id: string) => {
    const cur = tabsRef.current;
    const tab = cur.find((x) => x.id === id);
    if (!tab) return;
    const gone = new Set([id, ...cur.filter((x) => x.originId === id).map((x) => x.id)]); // fechar a origem leva o marcador
    if (tab.kind === "session") void api.closeTab(id).catch(() => {});
    let rest = cur.filter((x) => !gone.has(x.id));
    if (!rest.some((x) => x.kind === "session")) rest = [newSession(), ...rest];
    const idx = cur.findIndex((x) => x.id === id);
    if (gone.has(activeId)) {
      const next = rest.find((x) => cur.findIndex((y) => y.id === x.id) > idx) ?? rest[rest.length - 1];
      setActiveId(next.id);
    }
    setTabs(rest);
  }, [activeId]);

  const titleOf = (t: Tab) => t.kind === "browser" ? (tabs.find((x) => x.id === t.originId)?.title ?? "") : t.title;
  const infos: TabInfo[] = tabs.map((t) => ({ id: t.id, title: titleOf(t), kind: t.kind }));
  const sessions = tabs.filter((t) => t.kind === "session").length;
  const active = tabs.find((t) => t.id === activeId) ?? tabs[0];

  return (
    <div style={{ height: "100vh", display: "flex", flexDirection: "column", overflow: "hidden" }}>
      <UpdateBar />
      <UsageBar usage={usage} />
      <TabBar tabs={infos} activeId={active.id} onSelect={setActiveId} onClose={closeTab} onNew={addTab}
        reserveRight={Object.keys(usage).length ? 330 : 0} />
      <div style={{ flex: 1, minHeight: 0, position: "relative" }}>
        {tabs.map((t) => (
          <div key={t.id} style={{ position: "absolute", inset: 0, display: t.id === active.id ? "block" : "none" }}>
            {t.kind === "session" ? (
              <TabView tabId={t.id} active={t.id === active.id} canQuit={sessions === 1} dir={dir} options={options} browserOptions={browserOptions}
                onTitle={(title) => setTitle(t.id, title)} onReset={() => dropMarker(t.id)} onBrowserOpen={() => openMarker(t.id)} />
            ) : (
              <BrowserTab originTitle={titleOf(t)} onBack={() => setActiveId(t.originId!)} />
            )}
          </div>
        ))}
      </div>
    </div>
  );
}
