
import express from "express";
import cors from "cors";
import bodyParser from "body-parser";
import { buildIndex } from "./search/indexer";
import { retrieve } from "./search/retriever";
import { rank } from "./search/ranker";
import { loadDocs } from "./sources/docs";
import { loadSchemas } from "./sources/schemas";
import { loadExamples } from "./sources/examples";
import { answer } from "./handlers/what_is_signia";
import { howToDeploy } from "./handlers/how_to_deploy";
import { troubleshooting } from "./handlers/troubleshooting";

type Doc = { id: string; title: string; text: string; tags: string[] };

const PORT = Number(process.env.PORT || 7070);
const REPO_ROOT = process.env.SIGNIA_REPO_ROOT || process.cwd();

async function main() {
  const app = express();
  app.use(cors());
  app.use(bodyParser.json({ limit: "1mb" }));

  const docs: Doc[] = [
    ...(await loadDocs(REPO_ROOT)),
    ...(await loadSchemas(REPO_ROOT)),
    ...(await loadExamples(REPO_ROOT)),
  ];

  const index = buildIndex(docs);

  app.get("/healthz", (_req, res) => res.json({ ok: true }));

  app.post("/ask", async (req, res) => {
    const query = String(req.body?.query || "").trim();
    if (!query) return res.status(400).json({ ok: false, error: "missing query" });

    // High-precision handlers first
    const direct =
      howToDeploy(query) ||
      troubleshooting(query) ||
      answer(query);

    const candidates = retrieve(index, query, 12);
    const ranked = rank(candidates, query).slice(0, 6);

    const response = {
      ok: true,
      query,
      direct,
      top: ranked.map((r) => ({ id: r.doc.id, title: r.doc.title, score: r.score, tags: r.doc.tags })),
      context: ranked.map((r) => ({ id: r.doc.id, excerpt: r.excerpt })),
      notes: [
        "This service is deterministic and does not call external LLMs.",
        "For deeper answers, extend sources under src/sources/ or add new handlers.",
      ],
    };

    res.json(response);
  });

  app.listen(PORT, () => {
    // eslint-disable-next-line no-console
    console.log(`interface listening on :${PORT}`);
  });
}

main().catch((e) => {
  // eslint-disable-next-line no-console
  console.error(e);
  process.exit(1);
});
