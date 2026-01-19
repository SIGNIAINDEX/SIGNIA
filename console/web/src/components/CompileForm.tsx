
"use client";

import { useMemo, useState } from "react";
import { compileStructure, getArtifact } from "../lib/api";
import SchemaViewer from "./SchemaViewer";
import ProofViewer from "./ProofViewer";
import ArtifactList, { ArtifactRow } from "./ArtifactList";

const DEFAULT_INPUT = `{
  "kind": "openapi",
  "source": {
    "type": "inline",
    "name": "petstore",
    "content": {
      "openapi": "3.0.0",
      "info": { "title": "Petstore", "version": "1.0.0" },
      "paths": { "/pets": { "get": { "responses": { "200": { "description": "ok" } } } } }
    }
  }
}`;

export default function CompileForm() {
  const [input, setInput] = useState(DEFAULT_INPUT);
  const [busy, setBusy] = useState(false);
  const [err, setErr] = useState<string | null>(null);
  const [result, setResult] = useState<any>(null);
  const [artifact, setArtifact] = useState<any>(null);

  const artifacts: ArtifactRow[] = useMemo(() => {
    if (!result?.artifacts) return [];
    return result.artifacts as ArtifactRow[];
  }, [result]);

  async function onCompile() {
    setErr(null);
    setArtifact(null);
    setBusy(true);
    try {
      const payload = JSON.parse(input);
      const out = await compileStructure(payload);
      setResult(out);
    } catch (e: any) {
      setErr(e?.message || String(e));
    } finally {
      setBusy(false);
    }
  }

  async function openArtifact(id: string) {
    setErr(null);
    try {
      const a = await getArtifact(id);
      setArtifact(a);
    } catch (e: any) {
      setErr(e?.message || String(e));
    }
  }

  return (
    <div className="row">
      <div className="col">
        <div className="panel">
          <div className="h2">Input</div>
          <p className="p">Paste a structure payload. The backend will canonicalize → detect → compile → produce schema/manifest/proof.</p>
          <textarea value={input} onChange={(e) => setInput(e.target.value)} />
          <div style={{ display: "flex", gap: 10, marginTop: 12, alignItems: "center" }}>
            <button className="btn primary" disabled={busy} onClick={onCompile}>
              {busy ? "Compiling…" : "Compile"}
            </button>
            {err ? <span style={{ color: "var(--danger)" }}>{err}</span> : <span className="small">API: /v1/compile</span>}
          </div>
        </div>

        <div style={{ marginTop: 12 }}>
          <ArtifactList items={artifacts} onPick={openArtifact} />
        </div>

        {artifact ? (
          <div className="panel" style={{ marginTop: 12 }}>
            <div className="h2">Artifact</div>
            <pre style={{ margin: 0, overflowX: "auto" }}>{JSON.stringify(artifact, null, 2)}</pre>
          </div>
        ) : null}
      </div>

      <div className="col">
        {result?.schema ? <SchemaViewer schema={result.schema} /> : <div className="panel"><div className="h2">Schema</div><div className="small">No result yet.</div></div>}
        <div style={{ height: 12 }} />
        {result?.proof ? <ProofViewer proof={result.proof} /> : <div className="panel"><div className="h2">Proof</div><div className="small">No result yet.</div></div>}
        <div style={{ height: 12 }} />
        {result?.manifest ? (
          <div className="panel">
            <div className="h2">Manifest</div>
            <pre style={{ margin: 0, overflowX: "auto" }}>{JSON.stringify(result.manifest, null, 2)}</pre>
          </div>
        ) : (
          <div className="panel">
            <div className="h2">Manifest</div>
            <div className="small">No result yet.</div>
          </div>
        )}
      </div>
    </div>
  );
}
