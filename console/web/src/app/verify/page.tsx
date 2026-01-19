
"use client";

import { useState } from "react";
import { verifyBundle } from "../../lib/api";

const DEFAULT_VERIFY = `{
  "schema": { "version": "v1", "kind": "example", "nodes": [], "edges": [] },
  "manifest": { "schemaHash": "deadbeef", "artifactHashes": [] },
  "proof": { "root": "deadbeef", "leaves": [] }
}`;

export default function VerifyPage() {
  const [payload, setPayload] = useState(DEFAULT_VERIFY);
  const [busy, setBusy] = useState(false);
  const [out, setOut] = useState<any>(null);
  const [err, setErr] = useState<string | null>(null);

  async function onVerify() {
    setBusy(true);
    setErr(null);
    setOut(null);
    try {
      const obj = JSON.parse(payload);
      const res = await verifyBundle(obj);
      setOut(res);
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
          <div className="h1">Verify</div>
          <p className="p">Verify schema/manifest/proof bundles against deterministic rules.</p>
          <textarea value={payload} onChange={(e) => setPayload(e.target.value)} />
          <div style={{ display: "flex", gap: 10, marginTop: 12, alignItems: "center" }}>
            <button className="btn primary" disabled={busy} onClick={onVerify}>
              {busy ? "Verifying…" : "Verify"}
            </button>
            {err ? <span style={{ color: "var(--danger)" }}>{err}</span> : <span className="small">API: /v1/verify</span>}
          </div>
        </div>
      </div>

      <div className="col">
        <div className="panel">
          <div className="h2">Result</div>
          {out ? <pre style={{ margin: 0, overflowX: "auto" }}>{JSON.stringify(out, null, 2)}</pre> : <div className="small">No result yet.</div>}
        </div>
      </div>
    </div>
  );
}
