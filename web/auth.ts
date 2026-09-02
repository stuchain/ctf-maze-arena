import type { NextAuthOptions } from 'next-auth';
import GitHubProvider from 'next-auth/providers/github';
import { serverEnv } from '@/lib/server-env';

const githubProvider = serverEnv.GITHUB_ID && serverEnv.GITHUB_SECRET
  ? [
      GitHubProvider({
        clientId: serverEnv.GITHUB_ID,
        clientSecret: serverEnv.GITHUB_SECRET,
      }),
    ]
  : [];

export const authOptions: NextAuthOptions = {
  providers: githubProvider,
  secret: serverEnv.NEXTAUTH_SECRET,
};
