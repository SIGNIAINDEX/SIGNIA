
import { SIGNIA_API } from "./config";

export type CompileResponse = {
  schema: unknown;
  manifest: unknown;
  proof: unknown;
  artifacts?: Array<{ id: string; size?: number; kind?: string }>;
};

export async function compileStructure(payload: unknown): Promise<CompileResponse> {
  const r = await fetch(`${SIGNIA_API}/v1/compile`, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify(payload),
  });
  if (!r.ok) {
    const text = await r.text();
    throw new Error(text || `compile failed: ${r.status}`);
  }
  return (await r.json()) as CompileResponse;
}

export async function verifyBundle(payload: unknown): Promise<{ ok: boolean; details?: unknown }> {
  const r = await fetch(`${SIGNIA_API}/v1/verify`, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify(payload),
  });
  if (!r.ok) {
    const text = await r.text();
    throw new Error(text || `verify failed: ${r.status}`);
  }
  return (await r.json()) as { ok: boolean; details?: unknown };
}

export async function listPlugins(): Promise<any> {
  const r = await fetch(`${SIGNIA_API}/v1/plugins`, { cache: "no-store" });
  if (!r.ok) throw new Error(`plugins failed: ${r.status}`);
  return r.json();
}

export async function getArtifact(id: string): Promise<any> {
  const r = await fetch(`${SIGNIA_API}/v1/artifacts/${encodeURIComponent(id)}`, { cache: "no-store" });
  if (!r.ok) throw new Error(`artifact failed: ${r.status}`);
  return r.json();
}

export async function registryStatus(): Promise<any> {
  const r = await fetch(`${SIGNIA_API}/v1/registry/status`, { cache: "no-store" });
  if (!r.ok) throw new Error(`registry status failed: ${r.status}`);
  return r.json();
}
