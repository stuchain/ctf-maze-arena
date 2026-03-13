export function createSolveStreamUrl(runId: string): string {
  const base = process.env.NEXT_PUBLIC_API_URL || "http://localhost:8080";
  const wsBase = base.replace(/^http/, "ws");
  return `${wsBase}/api/solve/stream?runId=${encodeURIComponent(runId)}`;
}
