import { useEffect, useState } from "react";
import type { Update } from "@tauri-apps/plugin-updater";

const IGNORED_KEY = "orchestra.ignoredUpdate";
const isTauri = typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
const get = () => { try { return localStorage.getItem(IGNORED_KEY); } catch { return null; } };
const set = (v: string) => { try { localStorage.setItem(IGNORED_KEY, v); } catch { /* sem storage: ignora só nesta execução */ } };

const btn: React.CSSProperties = { border: 0, borderRadius: 4, padding: "1px 10px", fontSize: 12, cursor: "pointer", color: "#fff" };

/** Barra fina no topo avisando de uma nova versão (checa ao abrir o app). */
export default function UpdateBar() {
  const [update, setUpdate] = useState<Update | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!isTauri) return;
    let alive = true;
    import("@tauri-apps/plugin-updater")
      .then(({ check }) => check())
      .then((u) => { if (alive && u && u.version !== get()) setUpdate(u); })
      .catch(() => { /* sem rede/endpoint: sem aviso */ });
    return () => { alive = false; };
  }, []);

  if (!update) return null;

  const install = async () => {
    setBusy(true); setError(null);
    try {
      await update.downloadAndInstall();
      const { relaunch } = await import("@tauri-apps/plugin-process");
      await relaunch();
    } catch (e) {
      setError(`Falha ao atualizar: ${e}. Baixe em github.com/hugorochaffs/orquestrador/releases`);
      setBusy(false);
    }
  };
  const ignore = () => { set(update.version); setUpdate(null); };

  return (
    <div style={{ display: "flex", gap: 10, alignItems: "center", padding: "2px 12px", fontSize: 12, background: error ? "#7f1d1d" : "#1e3a8a", color: "#e5e7eb", flex: "none" }}>
      <span style={{ flex: 1 }}>{error ?? `Nova versão ${update.version} disponível${busy ? " — atualizando…" : ""}`}</span>
      {!busy && <button onClick={install} style={{ ...btn, background: "#2563eb" }}>Atualizar</button>}
      {!busy && <button onClick={ignore} style={{ ...btn, background: "#374151" }}>Ignorar</button>}
    </div>
  );
}
