import { Globe } from "lucide-react";

/** Aba marcador: o navegador (Claude in Chrome) é externo; aqui só se informa que a aba de origem o está usando. */
export default function BrowserTab({ originTitle, onBack }: { originTitle: string; onBack: () => void }) {
  return (
    <div style={{ height: "100%", display: "flex", alignItems: "center", justifyContent: "center", flexDirection: "column", gap: 14, color: "#e5e7eb", textAlign: "center", padding: 24 }}>
      <Globe size={40} color="#3b82f6" />
      <div style={{ fontSize: 18, fontWeight: 600 }}>Navegador em uso</div>
      <div style={{ fontSize: 13, color: "#9ca3af", maxWidth: 440 }}>
        A aba “{originTitle}” está usando o Chrome (Claude in Chrome). O navegador é externo ao Orchestra e abre em outra janela;
        fechar esta aba não afeta o Chrome.
      </div>
      <button onClick={onBack} style={{ background: "#2563eb", color: "#fff", border: 0, borderRadius: 6, padding: "8px 16px", fontSize: 14, cursor: "pointer" }}>
        Voltar à aba de origem
      </button>
    </div>
  );
}
