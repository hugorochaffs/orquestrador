import { useState } from "react";
import { ChevronDown, Plus, X } from "lucide-react";

export interface TabInfo { id: string; title: string; kind: "session" | "browser" }

interface Props {
  tabs: TabInfo[]; activeId: string; onSelect: (id: string) => void; onClose: (id: string) => void; onNew: () => void;
  /** Espaço reservado à direita (barra de uso fixa). */
  reserveRight: number;
}

/** Barra de abas estilo GNOME Terminal: título com reticências, × por aba, "+" e menu com todas as abas. */
export default function TabBar({ tabs, activeId, onSelect, onClose, onNew, reserveRight }: Props) {
  const [menu, setMenu] = useState(false);
  return (
    <div style={{ display: "flex", alignItems: "stretch", height: 34, flex: "none", background: "#12151c", borderBottom: "1px solid #1f2430", paddingRight: reserveRight, position: "relative" }}>
      <div style={{ display: "flex", overflowX: "auto", minWidth: 0 }}>
        {tabs.map((t) => {
          const on = t.id === activeId;
          return (
            <div key={t.id} onClick={() => onSelect(t.id)} title={t.title}
              onAuxClick={(e) => { if (e.button === 1) onClose(t.id); }}
              style={{ display: "flex", alignItems: "center", gap: 6, padding: "0 8px 0 12px", minWidth: 110, maxWidth: 200, cursor: "pointer", fontSize: 13, flex: "none",
                color: on ? "#e5e7eb" : "#9ca3af", background: on ? "#0b0d12" : "transparent", borderBottom: `2px solid ${on ? "#3b82f6" : "transparent"}` }}>
              <span style={{ overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap", flex: 1 }}>{t.kind === "browser" ? "🌐 " : ""}{t.title}</span>
              <button title="Fechar aba" onClick={(e) => { e.stopPropagation(); onClose(t.id); }}
                style={{ background: "transparent", border: 0, color: "inherit", cursor: "pointer", display: "flex", padding: 2, borderRadius: 4 }}><X size={14} /></button>
            </div>
          );
        })}
      </div>
      <button title="Nova aba" onClick={onNew} style={{ background: "transparent", border: 0, color: "#9ca3af", cursor: "pointer", padding: "0 10px" }}><Plus size={16} /></button>
      <button title="Todas as abas" onClick={() => setMenu((m) => !m)} style={{ background: "transparent", border: 0, color: "#9ca3af", cursor: "pointer", padding: "0 8px" }}><ChevronDown size={16} /></button>
      {menu && (
        <div onMouseLeave={() => setMenu(false)} style={{ position: "absolute", top: 34, right: reserveRight, zIndex: 60, background: "#12151c", border: "1px solid #2a3040", borderRadius: 6, minWidth: 220, maxWidth: 320, padding: 4 }}>
          {tabs.map((t) => (
            <div key={t.id} onClick={() => { onSelect(t.id); setMenu(false); }}
              style={{ padding: "6px 10px", fontSize: 13, cursor: "pointer", borderRadius: 4, color: t.id === activeId ? "#60a5fa" : "#e5e7eb", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
              {t.kind === "browser" ? "🌐 " : ""}{t.title}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
