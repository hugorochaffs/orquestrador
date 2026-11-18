import type { Edge, Node } from "@xyflow/react";
import type { Plan } from "../types";

export const NODE_W = 520;
export const NODE_H = 340;

export function layoutPlan(plan: Plan): { nodes: Node[]; edges: Edge[] } {
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
  const rows = new Map<number, number>();
  const nodes: Node[] = plan.tasks.map((t) => {
    const l = lv(t.id);
    const i = rows.get(l) ?? 0;
    rows.set(l, i + 1);
    return {
      id: t.id,
      type: "terminal",
      position: { x: i * (NODE_W + 60), y: l * (NODE_H + 80) },
      data: { title: t.title, status: "pending", ptyId: null },
    };
  });
  const edges: Edge[] = plan.tasks.flatMap((t) =>
    t.depends_on.map((d) => ({ id: `${d}->${t.id}`, source: d, target: t.id, animated: true })),
  );
  return { nodes, edges };
}
