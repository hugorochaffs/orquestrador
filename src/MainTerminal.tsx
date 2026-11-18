import { useEffect, useRef } from "react";
import { Terminal } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import { api } from "./api";

export const b64 = (s: string) => Uint8Array.from(atob(s), (c) => c.charCodeAt(0));

interface Props {
  ptyId: string | null;
  /** Saída já filtrada por pty_id que chegou antes do xterm montar. */
  pending: Uint8Array[];
  /** Registra o destino de escrita ao vivo. */
  register: (write: ((d: Uint8Array) => void) | null) => void;
  /** Muda quando o painel é redimensionado, para refazer o fit. */
  resizeKey: number;
}

export default function MainTerminal({ ptyId, pending, register, resizeKey }: Props) {
  const host = useRef<HTMLDivElement>(null);
  const term = useRef<Terminal | null>(null);
  const fit = useRef<FitAddon | null>(null);
  const idRef = useRef(ptyId);
  idRef.current = ptyId;

  const doFit = () => {
    const t = term.current;
    if (!t || !fit.current || !host.current || host.current.clientHeight === 0) return;
    fit.current.fit();
    if (idRef.current) api.resizePty(idRef.current, t.cols, t.rows).catch(() => {});
  };

  useEffect(() => {
    const t = new Terminal({ fontSize: 14, cursorBlink: true, theme: { background: "#0b0d11", foreground: "#e5e7eb" } });
    const f = new FitAddon();
    t.loadAddon(f);
    t.open(host.current!);
    term.current = t;
    fit.current = f;
    t.onData((d) => { if (idRef.current) api.writePty(idRef.current, d).catch(() => {}); });
    pending.forEach((d) => t.write(d));
    pending.length = 0;
    register((d) => t.write(d));
    const ro = new ResizeObserver(doFit);
    ro.observe(host.current!);
    doFit();
    t.focus();
    return () => { ro.disconnect(); register(null); t.dispose(); term.current = null; };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  useEffect(doFit, [ptyId, resizeKey]); // eslint-disable-line react-hooks/exhaustive-deps

  return <div ref={host} style={{ width: "100%", height: "100%", padding: 4, boxSizing: "border-box", background: "#0b0d11" }} />;
}
