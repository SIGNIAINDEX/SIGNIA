
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

export async function loadExamples(repoRoot: string): Promise<Doc[]> {
  const out: Doc[] = [];
  const candidates = [
    path.join(repoRoot, "schemas", "v1", "examples", "openapi.schema.json"),
    path.join(repoRoot, "schemas", "v1", "examples", "repo.schema.json"),
    path.join(repoRoot, "docs", "schemas", "examples.md"),
  ];

  for (const p of candidates) {
    const content = readIfExists(p);
    if (!content) continue;
    const rel = path.relative(repoRoot, p);
    out.push({
      id: `examples:${rel}`,
      title: `Example: ${rel}`,
      text: content,
      tags: ["examples", ...rel.split(/[\/\.]/g).filter(Boolean)],
    });
  }
  return out;
}
