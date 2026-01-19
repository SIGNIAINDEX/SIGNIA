// console_smoke.ts
//
// E2E smoke for Console web + Interface service.
// This script is intentionally dependency-light.
// Run it from repo root after starting the stack.
//
// Usage:
//   node tests/e2e/console_smoke.ts
//
// Environment:
//   SIGNIA_CONSOLE_URL (default http://127.0.0.1:3000)
//   SIGNIA_INTERFACE_URL (default http://127.0.0.1:7071)

const consoleUrl = process.env.SIGNIA_CONSOLE_URL ?? "http://127.0.0.1:3000";
const interfaceUrl = process.env.SIGNIA_INTERFACE_URL ?? "http://127.0.0.1:7071";

async function mustFetch(url: string, init?: RequestInit) {
  const res = await fetch(url, init);
  if (!res.ok) {
    const txt = await res.text().catch(() => "");
    throw new Error(`request failed: ${res.status} ${url}\n${txt}`);
  }
  return res;
}

async function main() {
  await mustFetch(`${consoleUrl}/`);
  await mustFetch(`${consoleUrl}/compile`);
  await mustFetch(`${consoleUrl}/verify`);

  const q = {
    question: "What is SIGNIA?",
    topK: 5
  };

  const res = await mustFetch(`${interfaceUrl}/ask`, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify(q),
  });
  const data = await res.json();
  if (!data || typeof data.answer !== "string") {
    throw new Error("invalid interface response");
  }
  console.log("ok");
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
