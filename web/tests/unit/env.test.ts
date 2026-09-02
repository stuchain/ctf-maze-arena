import { describe, expect, it } from 'vitest';
import { parsePublicEnv } from '@/lib/env';
import { parseServerEnv } from '@/lib/server-env';

describe('environment validation', () => {
  it('provides the local API default', () => {
    expect(parsePublicEnv({})).toMatchObject({
      NEXT_PUBLIC_API_URL: 'http://localhost:8080',
      NEXT_PUBLIC_AUTH_MODE: 'anonymous',
    });
  });

  it('rejects malformed public API URLs', () => {
    expect(() => parsePublicEnv({ NEXT_PUBLIC_API_URL: 'not-a-url' })).toThrow();
  });

  it('allows secret-free anonymous mode', () => {
    expect(parseServerEnv({ AUTH_MODE: 'anonymous' }).AUTH_MODE).toBe('anonymous');
  });

  it('requires all auth settings when JWT auth is enabled', () => {
    expect(() => parseServerEnv({ AUTH_MODE: 'jwt' })).toThrow();
  });

  it('accepts a complete JWT and GitHub configuration', () => {
    const secret = 'x'.repeat(32);
    expect(parseServerEnv({
      AUTH_MODE: 'jwt',
      GITHUB_ID: 'github-id',
      GITHUB_SECRET: secret,
      NEXTAUTH_SECRET: secret,
      NEXTAUTH_URL: 'https://example.vercel.app',
      JWT_SECRET: secret,
    }).AUTH_MODE).toBe('jwt');
  });
});
