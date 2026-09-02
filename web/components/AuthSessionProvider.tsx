'use client';

import { SessionProvider } from 'next-auth/react';
import { publicEnv } from '@/lib/env';

export function AuthSessionProvider({ children }: { children: React.ReactNode }) {
  const authDisabled = publicEnv.NEXT_PUBLIC_AUTH_MODE === 'anonymous';
  return (
    <SessionProvider
      session={authDisabled ? null : undefined}
      refetchOnWindowFocus={!authDisabled}
    >
      {children}
    </SessionProvider>
  );
}
