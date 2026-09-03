import type { DefaultSession } from 'next-auth';

declare module 'next-auth' {
  interface Session {
    user?: DefaultSession['user'] & {
      githubSubject?: string;
    };
  }
}

declare module 'next-auth/jwt' {
  interface JWT {
    githubSubject?: string;
  }
}
