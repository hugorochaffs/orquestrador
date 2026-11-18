import { useEffect, useState, type ReactNode } from "react";
import type { UsageUpdate } from "./types";

interface ProviderInfo { name: string; color: string; icon: ReactNode }

const mono = (letter: string, color: string): ReactNode => (
  <svg width="16" height="16" viewBox="0 0 16 16" aria-hidden>
    <circle cx="8" cy="8" r="8" fill={color} />
    <text x="8" y="11.5" textAnchor="middle" fontSize="10" fontWeight="700" fill="#fff" fontFamily="system-ui, sans-serif">{letter}</text>
  </svg>
);

// Mapa extensível: adicione novos providers aqui.
const PROVIDERS: Record<string, ProviderInfo> = {
  claude: {
    name: "Claude", color: "#D97757",
    icon: (
      <svg width="16" height="16" viewBox="0 0 16 16" aria-hidden>
        <g stroke="#D97757" strokeWidth="1.6" strokeLinecap="round">
          {Array.from({ length: 8 }, (_, i) => {
            const a = (i * Math.PI) / 4;
            return <line key={i} x1={8 + Math.cos(a) * 2} y1={8 + Math.sin(a) * 2} x2={8 + Math.cos(a) * 7} y2={8 + Math.sin(a) * 7} />;
          })}
        </g>
      </svg>
    ),
  },
  antigravity: { name: "Antigravity", color: "#4285F4", icon: mono("A", "#4285F4") },
  gemini: { name: "Gemini", color: "#8E75F0", icon: mono("G", "#8E75F0") },
  codex: { name: "Codex", color: "#10A37F", icon: mono("C", "#10A37F") },
};
const fallback = (p: string): ProviderInfo => ({
  name: p, color: "#6b7280", icon: mono((p[0] ?? "?").toUpperCase(), "#6b7280"),
});

const barColor = (pct: number) => (pct < 60 ? "#22c55e" : pct <= 85 ? "#eab308" : "#ef4444");

function resetText(at: number | null, now: number): string {
  if (at == null) return "reinício desconhecido";
  const s = Math.max(0, at - Math.floor(now / 1000));
  const d = Math.floor(s / 86400), h = Math.floor((s % 86400) / 3600), m = Math.floor((s % 3600) / 60);
  return `reinicia em ${d > 0 ? `${d}d ` : ""}${h}h ${m}m`;
}

function Meter({ label, pct, resetsAt, now }: { label: string; pct: number | null; resetsAt: number | null; now: number }) {
  const v = pct == null ? null : Math.min(100, Math.max(0, pct));
  return (
    <div title={`${label}: ${v == null ? "—" : Math.round(v) + "%"} — ${resetText(resetsAt, now)}`}
      style={{ display: "flex", alignItems: "center", gap: 4 }}>
      <span style={{ color: "#9ca3af" }}>{label}</span>
      <span style={{ width: 34, height: 4, borderRadius: 2, background: "#2a3040", overflow: "hidden", flex: "none" }}>
        <span style={{ display: "block", height: "100%", width: `${v ?? 0}%`, background: v == null ? "transparent" : barColor(v), transition: "width 0.3s" }} />
      </span>
      <span style={{ minWidth: 26, textAlign: "right", fontVariantNumeric: "tabular-nums" }}>{v == null ? "—" : `${Math.round(v)}%`}</span>
    </div>
  );
}

export default function UsageBar({ usage }: { usage: Record<string, UsageUpdate> }) {
  const [now, setNow] = useState(() => Date.now());
  useEffect(() => { const t = setInterval(() => setNow(Date.now()), 30000); return () => clearInterval(t); }, []);
  const list = Object.values(usage);
  if (!list.length) return null;
  return (
    <div style={{ position: "fixed", top: 4, right: 14, zIndex: 50, display: "flex", gap: 6, pointerEvents: "none", fontSize: 11, color: "#e5e7eb" }}>
      {list.map((u) => {
        const info = PROVIDERS[u.provider] ?? fallback(u.provider);
        return (
          <div key={u.provider} title={info.name}
            style={{ pointerEvents: "auto", display: "flex", alignItems: "center", gap: 8, padding: "2px 8px", borderRadius: 999,
              background: "rgba(17,20,28,0.88)", border: `1px solid ${info.color}55`, backdropFilter: "blur(4px)" }}>
            {info.icon}
            <Meter label="Sessão" pct={u.session_pct} resetsAt={u.session_resets_at} now={now} />
            <Meter label="Semana" pct={u.week_pct} resetsAt={u.week_resets_at} now={now} />
          </div>
        );
      })}
    </div>
  );
}
