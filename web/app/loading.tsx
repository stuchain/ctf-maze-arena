import { Panel, Skeleton } from '@/components/ui/Primitives';

export default function Loading() {
  return (
    <main id="main-content" className="loading-layout" aria-busy="true" aria-label="Loading arena">
      <Panel className="loading-card">
        <Skeleton className="skeleton--label" />
        <Skeleton className="skeleton--title" />
        <Skeleton className="skeleton--stage" />
        <span className="visually-hidden">Loading arena…</span>
      </Panel>
    </main>
  );
}
