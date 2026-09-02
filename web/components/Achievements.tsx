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
    <div className="space-y-2 w-full max-w-md">
      <h3 className="text-sm font-semibold text-zinc-700">Achievements</h3>
      {ACHIEVEMENTS.map((a) => {
        const isEarned = earned.has(a.id);
        return (
          <div
            key={a.id}
            className={`p-2 rounded border ${
              isEarned
                ? 'bg-amber-50 border-amber-200 dark:bg-amber-950/30 dark:border-amber-800'
                : 'bg-gray-50 border-gray-200 opacity-60 dark:bg-zinc-900 dark:border-zinc-700'
            }`}
          >
            <span className="font-medium">{a.name}</span>
            {isEarned ? ' ✓' : null}
            <p className="text-sm text-gray-600 dark:text-zinc-400">{a.description}</p>
          </div>
        );
      })}
    </div>
  );
}
