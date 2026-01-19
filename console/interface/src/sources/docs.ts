
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

function mdTitle(md: string, fallback: string): string {
  const m = md.match(/^#\s+(.+)$/m);
  return (m?.[1] || fallback).trim();
}

export async function loadDocs(repoRoot: string): Promise<Doc[]> {
  const docsRoot = path.join(repoRoot, "docs");
  if (!fs.existsSync(docsRoot)) return [];

  const candidates = [
    "overview.md",
    "architecture.md",
    "faq.md",
    "roadmap.md",
    "index.md",
    "api/openapi.yaml",
    "api/auth.md",
    "api/rate-limits.md",
    "api/error-codes.md",
    "cli/installation.md",
    "cli/usage.md",
    "cli/config.md",
    "cli/recipes.md",
  ];

  const out: Doc[] = [];
  for (const rel of candidates) {
    const p = path.join(docsRoot, rel);
    const content = readIfExists(p);
    if (!content) continue;
    const title = mdTitle(content, rel);
    out.push({
      id: `docs:${rel}`,
      title: `Docs: ${title}`,
      text: content,
      tags: ["docs", ...rel.split(/[\/\.]/g).filter(Boolean)],
    });
  }
  return out;
}
