
import { Scored } from "./retriever";

function phraseBoost(text: string, q: string): number {
  const t = text.toLowerCase();
  const qq = q.toLowerCase().trim();
  if (!qq) return 0;
  return t.includes(qq) ? 1.5 : 0;
}

export function rank(items: Scored[], query: string): Scored[] {
  return items
    .map((x) => ({
      ...x,
      score: x.score + phraseBoost(`${x.doc.title}\n${x.doc.text}`, query),
    }))
    .sort((a, b) => b.score - a.score);
}
