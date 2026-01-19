
"use client";

import { useState } from "react";
import { SIGNIA_INTERFACE } from "../../lib/config";

export default function InterfacePage() {
  const [q, setQ] = useState("How do I deploy SIGNIA locally?");
  const [busy, setBusy] = useState(false);
  const [out, setOut] = useState<any>(null);
  const [err, setErr] = useState<string | null>(null);

  async function ask() {
    setBusy(true);
    setErr(null);
    setOut(null);
    try {
      const r = await fetch(`${SIGNIA_INTERFACE}/ask`, {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ query: q }),
      });
      if (!r.ok) throw new Error(await r.text());
      setOut(await r.json());
    } catch (e: any) {
      setErr(e?.message || String(e));
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="row">
      <div className="col">
        <div className="panel">
          <div className="h1">Interface</div>
          <p className="p">
            The Interface service is a deterministic help layer. It indexes docs, schemas, and examples to answer project questions.
          </p>
          <input className="input" value={q} onChange={(e) => setQ(e.target.value)} />
          <div style={{ display: "flex", gap: 10, marginTop: 12, alignItems: "center" }}>
            <button className="btn primary" disabled={busy} onClick={ask}>{busy ? "Asking…" : "Ask"}</button>
            <span className="small">Service: {SIGNIA_INTERFACE}</span>
            {err ? <span style={{ color: "var(--danger)" }}>{err}</span> : null}
          </div>
        </div>
      </div>

      <div className="col">
        <div className="panel">
          <div className="h2">Answer</div>
          {out ? <pre style={{ margin: 0, overflowX: "auto" }}>{JSON.stringify(out, null, 2)}</pre> : <div className="small">No output yet.</div>}
        </div>
      </div>
    </div>
  );
}
