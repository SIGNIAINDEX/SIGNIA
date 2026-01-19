
import fs from "fs";
import path from "path";

type Doc = { id: string; title: string; text: string; tags: string[] };

function readIfExists(p: string): string | null {
  try {
    return fs.readFileSync(p, "utf8");
  } catch {
    return null;
  }
}

export async function loadSchemas(repoRoot: string): Promise<Doc[]> {
  const schemasRoot = path.join(repoRoot, "schemas");
  if (!fs.existsSync(schemasRoot)) return [];

  const candidates = [
    "README.md",
    "v1/schema.json",
    "v1/manifest.json",
    "v1/proof.json",
    "v1/dataset.schema.json",
    "v1/workflow.schema.json",
    "v1/examples/repo.schema.json",
    "v1/examples/openapi.schema.json",
  ];

  const out: Doc[] = [];
  for (const rel of candidates) {
    const p = path.join(schemasRoot, rel);
    const content = readIfExists(p);
    if (!content) continue;
    out.push({
      id: `schemas:${rel}`,
      title: `Schema: ${rel}`,
      text: content,
      tags: ["schemas", ...rel.split(/[\/\.]/g).filter(Boolean)],
    });
  }
  return out;
}
