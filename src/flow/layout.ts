import type { Edge, Node } from "@xyflow/react";
import type { Plan, TaskStatus } from "../types";

export const MIN_W = 420;
export const MIN_H = 260;
export const CW = 240; // card recolhido
export const CH = 44;
const GAP = 16;

export interface NodeInfo { status: TaskStatus; ptyId: string | null; reason?: string }

export function layoutPlan(
  plan: Plan,
  area: { w: number; h: number },
  collapsed: Set<string>,
  info: Record<string, NodeInfo>,
  onToggle: (id: string) => void,
): { nodes: Node[]; edges: Edge[] } {
  const byId = new Map(plan.tasks.map((t) => [t.id, t]));
  const level = new Map<string, number>();
  const lv = (id: string, seen: Set<string> = new Set()): number => {
    if (level.has(id)) return level.get(id)!;
    if (seen.has(id)) return 0;
    seen.add(id);
    const deps = (byId.get(id)?.depends_on ?? []).filter((d) => byId.has(d));
    const l = deps.length ? 1 + Math.max(...deps.map((d) => lv(d, seen))) : 0;
    level.set(id, l);
    return l;
  };
  const levels: string[][] = [];
  plan.tasks.forEach((t) => { (levels[lv(t.id)] ??= []).push(t.id); });
  const rows = levels.filter(Boolean);

  const isC = (id: string) => collapsed.has(id);
  const flex = rows.map((r) => r.some((id) => !isC(id)));
  const fixedH = rows.reduce((a, _, i) => a + (flex[i] ? 0 : CH), 0);
  const nFlex = flex.filter(Boolean).length;
  const totalGap = GAP * (rows.length + 1);
  const rowH = nFlex ? Math.max(MIN_H, (area.h - fixedH - totalGap) / nFlex) : 0;

  const nodes: Node[] = [];
  let y = GAP;
  rows.forEach((r, i) => {
    const h = flex[i] ? rowH : CH;
    const nc = r.filter(isC).length;
    const ne = r.length - nc;
    const w = ne ? Math.max(MIN_W, (area.w - GAP * (r.length + 1) - nc * CW) / ne) : 0;
    const total = r.reduce((a, id) => a + (isC(id) ? CW : w), 0) + GAP * (r.length - 1);
    let x = Math.max(GAP, (area.w - total) / 2);
    r.forEach((id) => {
      const c = isC(id);
      const nw = c ? CW : w;
      const nh = c ? CH : h;
      const inf = info[id] ?? { status: "pending" as TaskStatus, ptyId: null };
      nodes.push({
        id, type: "terminal", position: { x, y: c && flex[i] ? y + (h - CH) / 2 : y },
        style: { width: nw, height: nh },
        data: { title: byId.get(id)!.title, ...inf, compact: c, onToggle },
      });
      x += nw + GAP;
    });
    y += h + GAP;
  });
  const edges: Edge[] = plan.tasks.flatMap((t) =>
    t.depends_on.filter((d) => byId.has(d)).map((d) => ({
      id: `${d}->${t.id}`, source: d, target: t.id, animated: true,
      style: { stroke: "#94a3b8", strokeWidth: 2 }, markerEnd: { type: "arrowclosed" as const, color: "#94a3b8" },
    })),
  );
  return { nodes, edges };
}
