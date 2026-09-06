'use client';

import { useEffect, useMemo, useSyncExternalStore, useState } from 'react';
import { useSession } from 'next-auth/react';
import { authenticatedHeaders } from '@/lib/auth-api';
import { profileSchema, requestJson, type UserProfile } from '@/lib/api';
import { publicEnv } from '@/lib/env';
import { Notice, Skeleton } from '@/components/ui/Primitives';
import {
  ACHIEVEMENTS,
  EMPTY_ACHIEVEMENTS_SNAPSHOT,
  getAchievementsSnapshot,
  subscribeToAchievements,
} from '@/lib/achievements';

export function Achievements({ refreshKey }: { refreshKey?: string | null }) {
  const { status } = useSession();
  const [profile, setProfile] = useState<UserProfile | null>(null);
  const [error, setError] = useState('');
  const snapshot = useSyncExternalStore(
    subscribeToAchievements,
    getAchievementsSnapshot,
    () => EMPTY_ACHIEVEMENTS_SNAPSHOT,
  );
  const legacy = useMemo(() => new Set<string>(JSON.parse(snapshot)), [snapshot]);

  useEffect(() => {
    if (status !== 'authenticated') return;
    let active = true;
    authenticatedHeaders(false)
      .then((headers) => requestJson(`${process.env.NEXT_PUBLIC_API_URL}/api/profile`, profileSchema, { headers }))
      .then((data) => { if (active) setProfile(data); })
      .catch(() => { if (active) setError('Persistent achievements are temporarily unavailable.'); });
    return () => { active = false; };
  }, [status, refreshKey]);

  if (status === 'authenticated' && !profile && !error) return <div className="achievement-list" aria-label="Loading achievements"><Skeleton /><Skeleton /></div>;
  if (error) return <Notice title="Achievements unavailable" tone="warning">{error}</Notice>;

  if (status === 'authenticated') {
    const awards = profile?.achievements ?? [];
    return awards.length ? (
      <div className="achievement-list">
        {awards.map((award) => (
          <div key={`${award.key}:${award.version}`} className="achievement achievement--earned">
            <span className="achievement__icon" aria-hidden="true">◆</span>
            <div><strong>{award.name}</strong><p>{award.description}</p></div>
            <span className="achievement__state">Earned · v{award.version}</span>
          </div>
        ))}
      </div>
    ) : <p className="community-hint">Submit an authenticated run to earn durable achievements.</p>;
  }

  return (
    <div className="achievement-list">
      <p className="community-hint">{publicEnv.NEXT_PUBLIC_AUTH_MODE === 'anonymous' ? 'Persistent identity is disabled here.' : 'Sign in to earn durable achievements.'} Browser-only progress is marked legacy and is never promoted.</p>
      {ACHIEVEMENTS.map((a) => {
        const isEarned = legacy.has(a.id);
        return (
          <div
            key={a.id}
            className={`achievement ${isEarned ? 'achievement--earned' : ''}`}
          >
            <span className="achievement__icon" aria-hidden="true">{isEarned ? '◆' : '◇'}</span>
            <div>
              <strong>{a.name}</strong>
              <p>{a.description}</p>
            </div>
            <span className="achievement__state">{isEarned ? 'Legacy local' : 'Locked'}</span>
          </div>
        );
      })}
    </div>
  );
}
