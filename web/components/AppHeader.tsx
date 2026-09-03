'use client';

import { signIn, signOut, useSession } from 'next-auth/react';
import Link from 'next/link';
import { BrandMark } from '@/components/BrandMark';
import { ThemeToggle } from '@/components/ThemeToggle';
import { Button } from '@/components/ui/Primitives';
import { publicEnv } from '@/lib/env';

export function AppHeader() {
  const { data: session, status } = useSession();
  const authEnabled = publicEnv.NEXT_PUBLIC_AUTH_MODE !== 'anonymous';

  return (
    <header className="app-header">
      <Link className="brand" href="/" aria-label="CTF Maze Arena home" translate="no">
        <BrandMark />
        <span className="brand__copy">
          <strong>CTF Maze Arena</strong>
          <small>Pathfinding Lab</small>
        </span>
      </Link>

      <nav className="header-actions" aria-label="Global navigation">
        <a
          className="repo-link"
          href="https://github.com/stuchain/ctf-maze-arena"
          target="_blank"
          rel="noreferrer"
        >
          <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M12 2a10 10 0 0 0-3.2 19.5c.5.1.7-.2.7-.5v-1.9c-2.8.6-3.4-1.2-3.4-1.2-.5-1.2-1.1-1.5-1.1-1.5-.9-.6.1-.6.1-.6 1 0 1.6 1 1.6 1 .9 1.6 2.4 1.1 3 .8.1-.7.4-1.1.7-1.4-2.3-.3-4.6-1.1-4.6-5a3.9 3.9 0 0 1 1-2.7 3.6 3.6 0 0 1 .1-2.7s.8-.3 2.8 1A9.5 9.5 0 0 1 12 6.4a9.5 9.5 0 0 1 2.5.4c2-1.3 2.8-1 2.8-1a3.6 3.6 0 0 1 .1 2.7 3.9 3.9 0 0 1 1 2.7c0 3.9-2.4 4.7-4.6 5 .4.3.7 1 .7 2V21c0 .3.2.6.7.5A10 10 0 0 0 12 2Z" /></svg>
          <span>GitHub</span>
        </a>
        <ThemeToggle />
        {!authEnabled ? (
          <span className="anonymous-label">Guest Mode</span>
        ) : status === 'authenticated' ? (
          <Button variant="secondary" size="sm" onClick={() => void signOut()}>
            Sign Out <span className="header-user">· {session?.user?.name ?? 'GitHub User'}</span>
          </Button>
        ) : (
          <Button variant="secondary" size="sm" loading={status === 'loading'} onClick={() => void signIn('github')}>
            Sign In
          </Button>
        )}
      </nav>
    </header>
  );
}
