import { z } from 'zod';

const optionalSecret = z.string().min(32).optional();

const serverEnvSchema = z.object({
  AUTH_MODE: z.enum(['anonymous', 'optional_jwt', 'jwt']).default('anonymous'),
  GITHUB_ID: z.string().min(1).optional(),
  GITHUB_SECRET: optionalSecret,
  NEXTAUTH_SECRET: optionalSecret,
  NEXTAUTH_URL: z.string().url().optional(),
  JWT_SECRET: optionalSecret,
}).superRefine((env, context) => {
  if (env.AUTH_MODE === 'anonymous') return;
  for (const key of ['GITHUB_ID', 'GITHUB_SECRET', 'NEXTAUTH_SECRET', 'NEXTAUTH_URL', 'JWT_SECRET'] as const) {
    if (!env[key]) {
      context.addIssue({
        code: 'custom',
        path: [key],
        message: `${key} is required when AUTH_MODE is ${env.AUTH_MODE}`,
      });
    }
  }
});

export function parseServerEnv(input: Record<string, string | undefined>) {
  return serverEnvSchema.parse(input);
}

export const serverEnv = parseServerEnv({
  AUTH_MODE: process.env.AUTH_MODE,
  GITHUB_ID: process.env.GITHUB_ID,
  GITHUB_SECRET: process.env.GITHUB_SECRET,
  NEXTAUTH_SECRET: process.env.NEXTAUTH_SECRET,
  NEXTAUTH_URL: process.env.NEXTAUTH_URL,
  JWT_SECRET: process.env.JWT_SECRET,
});
