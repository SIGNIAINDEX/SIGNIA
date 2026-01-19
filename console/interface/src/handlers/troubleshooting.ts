
export function troubleshooting(query: string): { title: string; tips: string[] } | null {
  const q = query.toLowerCase();
  if (!(q.includes("error") || q.includes("failed") || q.includes("trouble") || q.includes("cannot"))) return null;

  const tips = [
    "Confirm API base URL: NEXT_PUBLIC_SIGNIA_API (default http://localhost:8080).",
    "Check Interface URL: NEXT_PUBLIC_SIGNIA_INTERFACE (default http://localhost:7070).",
    "If compile fails, ensure JSON is valid and includes a top-level 'kind' and 'source'.",
    "If verify fails, confirm the schema/manifest/proof are from the same compile output and were not edited.",
    "For registry issues, ensure Anchor localnet is running and program id matches Anchor.toml.",
  ];

  return { title: "Troubleshooting", tips };
}
