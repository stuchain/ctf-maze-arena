import Link from 'next/link';
import { AppHeader } from '@/components/AppHeader';
import { Badge, Panel } from '@/components/ui/Primitives';

export default function NotFound() {
  return (
    <div className="replay-shell">
      <AppHeader />
      <main id="main-content" className="state-page">
        <Panel className="state-card">
          <Badge tone="warning">404 · Dead end</Badge>
          <h1>No route reaches this page.</h1>
          <p>The destination may have moved, or the path was never part of this maze.</p>
          <Link href="/" className="button button--primary button--md">Return to the arena</Link>
        </Panel>
      </main>
    </div>
  );
}
