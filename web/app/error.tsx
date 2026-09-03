'use client';

import { useEffect } from 'react';
import { AppHeader } from '@/components/AppHeader';
import { Badge, Button, Panel } from '@/components/ui/Primitives';

export default function ErrorPage({ error, reset }: { error: Error & { digest?: string }; reset: () => void }) {
  useEffect(() => { console.error(error); }, [error]);

  return (
    <div className="replay-shell">
      <AppHeader />
      <main id="main-content" className="state-page">
        <Panel className="state-card">
          <Badge tone="danger">System fault</Badge>
          <h1>The arena hit an unexpected dead end.</h1>
          <p>Your configuration is still here. Try the request again, or return to the arena.</p>
          <Button onClick={reset}>Try again</Button>
        </Panel>
      </main>
    </div>
  );
}
