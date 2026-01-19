
export type Doc = { id: string; title: string; text: string; tags: string[] };

export type Index = {
  docs: Doc[];
  df: Map<string, number>;
  docTokens: Map<string, string[]>;
  N: number;
};

function tokenize(s: string): string[] {
  return s
    .toLowerCase()
    .replace(/[^a-z0-9\s-]/g, " ")
    .split(/\s+/)
    .filter((t) => t.length >= 2);
}

export function buildIndex(docs: Doc[]): Index {
  const df = new Map<string, number>();
  const docTokens = new Map<string, string[]>();

  for (const d of docs) {
    const tokens = tokenize(`${d.title}\n${d.text}`);
    docTokens.set(d.id, tokens);

    const seen = new Set(tokens);
    for (const t of seen) {
      df.set(t, (df.get(t) || 0) + 1);
    }
  }

  return { docs, df, docTokens, N: docs.length };
}

export function tokenizeQuery(q: string): string[] {
  return tokenize(q);
}
