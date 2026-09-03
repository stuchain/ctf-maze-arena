import { publicEnv } from '@/lib/env';

export function createSolveStreamUrl(runId: string, afterSequence = 0): string {
  const base = publicEnv.NEXT_PUBLIC_API_URL;
  const wsBase = base.replace(/^http/, 'ws');
  return `${wsBase}/api/solve/stream?runId=${encodeURIComponent(runId)}&afterSequence=${afterSequence}`;
}
