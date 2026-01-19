
"use client";

import { useEffect, useState } from "react";
import { listPlugins, registryStatus } from "../../lib/api";
import RegistryTable from "../../components/RegistryTable";

export default function RegistryPage() {
  const [status, setStatus] = useState<any>(null);
  const [plugins, setPlugins] = useState<any>(null);
  const [err, setErr] = useState<string | null>(null);

  useEffect(() => {
    (async () => {
      try {
        const [s, p] = await Promise.all([registryStatus(), listPlugins()]);
        setStatus(s);
        setPlugins(p);
      } catch (e: any) {
        setErr(e?.message || String(e));
      }
    })();
  }, []);

  return (
    <div className="row">
      <div className="col">
        <div className="panel">
          <div className="h1">Registry</div>
          <p className="p">On-chain registry status plus supported plugin types.</p>
          {err ? <div style={{ color: "var(--danger)" }}>{err}</div> : null}
        </div>
        <div style={{ height: 12 }} />
        <RegistryTable data={status} />
      </div>

      <div className="col">
        <div className="panel">
          <div className="h2">Plugins</div>
          {plugins ? (
            <pre style={{ margin: 0, overflowX: "auto" }}>{JSON.stringify(plugins, null, 2)}</pre>
          ) : (
            <div className="small">Loading…</div>
          )}
        </div>
      </div>
    </div>
  );
}
