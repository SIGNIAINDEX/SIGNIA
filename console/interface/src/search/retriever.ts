
import { Index, tokenizeQuery } from "./indexer";

export type Scored = {
  doc: { id: string; title: string; text: string; tags: string[] };
  score: number;
  excerpt: string;
};

function tf(tokens: string[], term: string): number {
  let c = 0;
  for (const t of tokens) if (t === term) c++;
  return c;
}

function idf(df: number, N: number): number {
  return Math.log(1 + N / (1 + df));
}

function excerpt(text: string, terms: string[]): string {
  const lower = text.toLowerCase();
  let bestIdx = -1;
  for (const t of terms) {
    const i = lower.indexOf(t);
    if (i !== -1 && (bestIdx === -1 || i < bestIdx)) bestIdx = i;
  }
  if (bestIdx === -1) return text.slice(0, 220);
  const start = Math.max(0, bestIdx - 60);
  return text.slice(start, start + 260);
}

export function retrieve(index: Index, query: string, limit = 10): Scored[] {
  const qTokens = tokenizeQuery(query);
  const unique = Array.from(new Set(qTokens));
  const out: Scored[] = [];

  for (const d of index.docs) {
    const tokens = index.docTokens.get(d.id) || [];
    let s = 0;

    for (const term of unique) {
      const dfv = index.df.get(term) || 0;
      const w = idf(dfv, index.N);
      s += tf(tokens, term) * w;
    }

    // small boost if tags match
    for (const t of unique) {
      if (d.tags.some((x) => x.toLowerCase() === t)) s += 0.35;
    }

    if (s > 0) out.push({ doc: d, score: s, excerpt: excerpt(d.text, unique) });
  }

  out.sort((a, b) => b.score - a.score);
  return out.slice(0, limit);
}
