import { z } from 'zod';

const publicEnvSchema = z.object({
  NEXT_PUBLIC_API_URL: z.string().url().default('http://localhost:8080'),
  NEXT_PUBLIC_AUTH_MODE: z.enum(['anonymous', 'optional_jwt', 'jwt']).default('anonymous'),
});

export function parsePublicEnv(input: Record<string, string | undefined>) {
  return publicEnvSchema.parse(input);
}

export const publicEnv = parsePublicEnv({
  NEXT_PUBLIC_API_URL: process.env.NEXT_PUBLIC_API_URL,
  NEXT_PUBLIC_AUTH_MODE: process.env.NEXT_PUBLIC_AUTH_MODE,
});
