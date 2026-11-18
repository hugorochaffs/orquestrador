import { useEffect } from "react";
import { X } from "lucide-react";
import type { SubagentLog } from "../types";
import { LogList, ModelBadge, StatusBadge } from "./SubagentNode";
import "./flow.css";

export interface ModalAgent {
  id: string; agentType: string; description: string | null;
  status: "running" | "done"; summary: string | null; logs: SubagentLog[]; model: string | null;
}

export default function SubagentModal({ agent, onClose }: { agent: ModalAgent; onClose: () => void }) {
  useEffect(() => {
    const k = (e: KeyboardEvent) => { if (e.key === "Escape") onClose(); };
    window.addEventListener("keydown", k);
    return () => window.removeEventListener("keydown", k);
  }, [onClose]);
  const done = agent.status === "done";
  return (
    <div className="sa-overlay" onMouseDown={(e) => { if (e.target === e.currentTarget) onClose(); }}>
      <div className={`sa sa-modal ${agent.status}`} role="dialog" aria-modal="true">
        <div className="sa-head">
          <span className="sa-title"><b>{agent.agentType}</b>{agent.description ? ` — ${agent.description}` : ""}</span>
          <ModelBadge model={agent.model} />
          <StatusBadge done={done} />
          <button className="sa-icon" title="Fechar (Esc)" onClick={onClose}><X size={16} /></button>
        </div>
        <LogList logs={agent.logs} big running={!done} className="sa-body sa-body-big" />
        {done && agent.summary && <div className="sa-foot">{agent.summary}</div>}
      </div>
    </div>
  );
}
