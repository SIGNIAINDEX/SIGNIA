
export function shortenPk(pk: string, n = 4): string {
  if (!pk) return "";
  if (pk.length <= n * 2 + 2) return pk;
  return `${pk.slice(0, n)}…${pk.slice(-n)}`;
}
