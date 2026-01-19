
export function answer(query: string): { title: string; body: string } | null {
  const q = query.toLowerCase();

  if (q.includes("what is signia") || q.includes("what's signia") || q.includes("define signia")) {
    return {
      title: "What is SIGNIA?",
      body:
        "SIGNIA is a structure-level compiler that converts real-world structures (repos, datasets, docs, API specs, workflows) into deterministic, verifiable forms. " +
        "It focuses on canonicalization, stable hashing, and proof generation so on-chain systems can reference structure identities without copying raw content.",
    };
  }

  if (q.includes("what does it do") || q.includes("core idea")) {
    return {
      title: "Core idea",
      body:
        "Take an input structure, canonicalize it, infer its structural graph, compile to a schema/manifest, and produce a proof. " +
        "The output is stable across machines and builds, making it suitable for registry publication and long-term referencing.",
    };
  }

  return null;
}
