'use client';

import { useMemo, useSyncExternalStore } from 'react';
import {
  ACHIEVEMENTS,
  EMPTY_ACHIEVEMENTS_SNAPSHOT,
  getAchievementsSnapshot,
  subscribeToAchievements,
} from '@/lib/achievements';

export function Achievements() {
  const snapshot = useSyncExternalStore(
    subscribeToAchievements,
    getAchievementsSnapshot,
    () => EMPTY_ACHIEVEMENTS_SNAPSHOT,
  );
  const earned = useMemo(() => new Set<string>(JSON.parse(snapshot)), [snapshot]);

  return (
    <div className="achievement-list">
      {ACHIEVEMENTS.map((a) => {
        const isEarned = earned.has(a.id);
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
            <span className="achievement__state">{isEarned ? 'Earned' : 'Locked'}</span>
          </div>
        );
      })}
    </div>
  );
}
