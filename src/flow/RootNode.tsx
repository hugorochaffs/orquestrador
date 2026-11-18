import { Handle, Position, type NodeProps } from "@xyflow/react";
import { Network } from "lucide-react";
import "./flow.css";

export default function RootNode({ data }: NodeProps) {
  const { running, done } = data as { running: number; done: number };
  return (
    <div className="sa-root">
      <Network size={18} color="#60a5fa" />
      <div>
        <div className="sa-rtitle">Orquestrador (sessão principal)</div>
        <div className="sa-rsub">{running} rodando / {done} concluídos</div>
      </div>
      <Handle type="source" position={Position.Bottom} />
    </div>
  );
}
