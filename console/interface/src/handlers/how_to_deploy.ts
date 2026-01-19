
export function howToDeploy(query: string): { title: string; steps: string[] } | null {
  const q = query.toLowerCase();
  if (!q.includes("deploy") && !q.includes("run locally") && !q.includes("local")) return null;

  const steps = [
    "Install Rust stable + Node.js LTS + Solana CLI (for program work).",
    "Build workspace: `cargo build --workspace`.",
    "Start the API: `cargo run -p signia-api` (or docker compose if provided).",
    "Start the Interface service: `cd console/interface && npm install && npm run dev`.",
    "Start the Console web: `cd console/web && npm install && npm run dev`.",
    "Open http://localhost:3000 and verify /compile and /verify flows.",
  ];

  return { title: "Local deployment", steps };
}
