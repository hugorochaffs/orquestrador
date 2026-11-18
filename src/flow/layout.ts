export interface Box { x: number; y: number; w: number; h: number }

const GAP = 16;
const ROOT_H = 64;
const MIN_W = 360;
const MIN_H = 220;
const COMPACT_W = 260;
const COMPACT_H = 56;
const PER_ROW = 4;

/** Posições da raiz (centrada no topo) e dos filhos em linhas abaixo dela. */
export function computeLayout(width: number, height: number, collapsed: boolean[]): { root: Box; children: Box[] } {
  const rootW = Math.min(420, Math.max(260, width - 2 * GAP));
  const root = { x: (width - rootW) / 2, y: GAP, w: rootW, h: ROOT_H };
  const n = collapsed.length;
  const perRow = Math.max(1, Math.min(PER_ROW, Math.floor((width - GAP) / (MIN_W + GAP))));
  const rows: number[][] = [];
  for (let i = 0; i < n; i += perRow) rows.push(Array.from({ length: Math.min(perRow, n - i) }, (_, k) => i + k));
  const top = root.y + ROOT_H + 40;
  const rowCollapsed = rows.map((r) => r.every((i) => collapsed[i]));
  const expandedRows = rowCollapsed.filter((c) => !c).length;
  const compactRows = rows.length - expandedRows;
  const avail = height - top - GAP - compactRows * (COMPACT_H + GAP) - Math.max(0, expandedRows - 1) * GAP;
  const rowH = expandedRows ? Math.max(MIN_H, avail / expandedRows) : 0;
  const children: Box[] = new Array(n);
  let y = top;
  rows.forEach((r, ri) => {
    const h = rowCollapsed[ri] ? COMPACT_H : rowH;
    const cols = r.length;
    const cellW = Math.max(MIN_W, (width - GAP * (cols + 1)) / cols);
    const totalW = cols * cellW + (cols - 1) * GAP;
    let x = Math.max(GAP, (width - totalW) / 2);
    for (const i of r) {
      children[i] = collapsed[i] ? { x, y, w: COMPACT_W, h: COMPACT_H } : { x, y, w: cellW, h };
      x += cellW + GAP;
    }
    y += h + GAP;
  });
  return { root, children };
}
