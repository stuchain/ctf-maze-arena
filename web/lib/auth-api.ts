import { requestJson, tokenResponseSchema } from '@/lib/api';

export async function authenticatedHeaders(contentType = true): Promise<Record<string, string>> {
  const token = await requestJson('/api/token', tokenResponseSchema);
  return {
    ...(contentType ? { 'Content-Type': 'application/json' } : {}),
    Authorization: `Bearer ${token.token}`,
  };
}
