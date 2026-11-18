// Buffers de saída por task, fora do estado React.
const buffers = new Map<string, Uint8Array[]>();
const subs = new Map<string, (c: Uint8Array) => void>();

export function decode(b64: string): Uint8Array {
  const s = atob(b64);
  const u = new Uint8Array(s.length);
  for (let i = 0; i < s.length; i++) u[i] = s.charCodeAt(i);
  return u;
}

export function pushOutput(taskId: string, chunk: Uint8Array) {
  const s = subs.get(taskId);
  if (s) s(chunk);
  else {
    const b = buffers.get(taskId) ?? [];
    b.push(chunk);
    buffers.set(taskId, b);
  }
}

export function subscribe(taskId: string, fn: (c: Uint8Array) => void) {
  buffers.get(taskId)?.forEach(fn);
  buffers.delete(taskId);
  subs.set(taskId, fn);
  return () => {
    if (subs.get(taskId) === fn) subs.delete(taskId);
  };
}
