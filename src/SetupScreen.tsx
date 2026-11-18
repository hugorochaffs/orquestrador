import { useEffect, useMemo, useRef, useState } from "react";
import type { BrowserOption, ModelOption } from "./types";

const PROVIDER_LABEL: Record<string, string> = { anthropic: "Anthropic" };
const label = (p: string) => PROVIDER_LABEL[p] ?? p;

interface Props {
  dir: string; options: ModelOption[]; browserOptions: BrowserOption[]; starting: boolean; error: string | null;
  onStart: (m: ModelOption, browser: boolean) => void;
}

const field: React.CSSProperties = { background: "#12151c", color: "#e5e7eb", border: "1px solid #2a3040", borderRadius: 6, padding: "8px 10px", fontSize: 14, width: "100%" };
const cap: React.CSSProperties = { fontSize: 11, color: "#9ca3af", textTransform: "uppercase", letterSpacing: 0.5, marginBottom: 4 };

export default function SetupScreen({ dir, options, browserOptions, starting, error, onStart }: Props) {
  const providers = useMemo(() => [...new Set(options.map((o) => o.provider))], [options]);
  const [provider, setProvider] = useState<string | null>(null);
  const [modelId, setModelId] = useState<string | null>(null);
  const [browserSel, setBrowserSel] = useState<Record<string, boolean>>({});
  const btn = useRef<HTMLButtonElement>(null);

  const prov = provider ?? (options.find((o) => o.default)?.provider ?? providers[0] ?? "");
  const models = options.filter((o) => o.provider === prov);
  const model = models.find((m) => m.id === modelId) ?? models.find((m) => m.default) ?? models[0];

  useEffect(() => { if (model) btn.current?.focus(); }, [!!model]); // eslint-disable-line react-hooks/exhaustive-deps

  const browserOpt = browserOptions.find((b) => b.provider === prov);
  const browserOn = browserOpt ? (browserSel[prov] ?? browserOpt.enabled) : false;
  const go = () => { if (model && !starting) onStart(model, browserOn); };

  return (
    <div onKeyDown={(e) => { if (e.key === "Enter" && !(e.target instanceof HTMLButtonElement)) go(); }}
      style={{ height: "100%", display: "flex", alignItems: "center", justifyContent: "center", background: "#0b0d12", color: "#e5e7eb" }}>
      <div style={{ width: 420, maxWidth: "90vw", display: "flex", flexDirection: "column", gap: 18 }}>
        <h1 style={{ margin: 0, fontSize: 28, fontWeight: 600 }}>Orchestra</h1>
        <div>
          <div style={cap}>Pasta do projeto</div>
          <div title={dir} style={{ ...field, fontFamily: "monospace", fontSize: 13, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>{dir || "…"}</div>
        </div>
        <div>
          <div style={cap}>Provider</div>
          <div style={{ display: "flex", gap: 6 }}>
            {providers.map((p) => (
              <button key={p} onClick={() => { setProvider(p); setModelId(null); }}
                style={{ ...field, width: "auto", cursor: "pointer", background: p === prov ? "#1f2a44" : "#12151c", borderColor: p === prov ? "#3b82f6" : "#2a3040" }}>{label(p)}</button>
            ))}
          </div>
        </div>
        <div>
          <div style={cap}>Modelo</div>
          <select value={model?.id ?? ""} onChange={(e) => setModelId(e.target.value)} style={field} disabled={!models.length}>
            {models.map((m) => <option key={m.id} value={m.id}>{m.label} ({m.id})</option>)}
          </select>
        </div>
        {browserOpt && (
          <label style={{ display: "flex", gap: 10, alignItems: "flex-start", cursor: "pointer" }}>
            <input type="checkbox" checked={browserOn} onChange={(e) => setBrowserSel((b) => ({ ...b, [prov]: e.target.checked }))} style={{ marginTop: 3 }} />
            <span style={{ display: "flex", flexDirection: "column" }}>
              <span style={{ fontSize: 14 }}>{browserOpt.label}</span>
              <span style={{ fontSize: 12, color: "#9ca3af" }}>{browserOpt.note}</span>
            </span>
          </label>
        )}
        {error && <div style={{ background: "#7f1d1d", color: "#fecaca", fontSize: 13, padding: "8px 10px", borderRadius: 6 }}>{error}</div>}
        <button ref={btn} onClick={go} disabled={!model || starting}
          style={{ background: "#2563eb", color: "#fff", border: 0, borderRadius: 6, padding: "10px 0", fontSize: 15, fontWeight: 600, cursor: starting ? "default" : "pointer", opacity: !model || starting ? 0.6 : 1 }}>
          {starting ? "Iniciando…" : "Iniciar"}
        </button>
      </div>
    </div>
  );
}
