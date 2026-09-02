export interface Achievement {
  id: string;
  name: string;
  description: string;
  check: (stats: { visited: number; cost: number; solver: string }) => boolean;
}

export const ACHIEVEMENTS: Achievement[] = [
  {
    id: 'efficient',
    name: 'Efficient',
    description: 'Solve with fewer than 100 visited nodes',
    check: (s) => s.visited < 100,
  },
  {
    id: 'astar_optimal',
    name: 'A* Optimal',
    description: 'Complete a solve with A*',
    check: (s) => s.solver === 'ASTAR',
  },
  {
    id: 'dp_keys',
    name: 'Key Master',
    description: 'Solve a keys/doors puzzle with DP',
    check: (s) => s.solver === 'DP_KEYS',
  },
];

const STORAGE_KEY = 'ctf-maze-achievements';
const CHANGE_EVENT = 'ctf-maze-achievements-changed';
export const EMPTY_ACHIEVEMENTS_SNAPSHOT = '[]';

export function getAchievementsSnapshot(): string {
  if (typeof window === 'undefined') return EMPTY_ACHIEVEMENTS_SNAPSHOT;
  const stored = localStorage.getItem(STORAGE_KEY);
  if (!stored) return EMPTY_ACHIEVEMENTS_SNAPSHOT;
  try {
    const parsed = JSON.parse(stored);
    return Array.isArray(parsed) ? stored : EMPTY_ACHIEVEMENTS_SNAPSHOT;
  } catch {
    return EMPTY_ACHIEVEMENTS_SNAPSHOT;
  }
}

export function subscribeToAchievements(onChange: () => void) {
  const handleStorage = (event: StorageEvent) => {
    if (event.key === STORAGE_KEY) onChange();
  };
  window.addEventListener('storage', handleStorage);
  window.addEventListener(CHANGE_EVENT, onChange);
  return () => {
    window.removeEventListener('storage', handleStorage);
    window.removeEventListener(CHANGE_EVENT, onChange);
  };
}

export function getEarnedAchievements(): string[] {
  try {
    return JSON.parse(getAchievementsSnapshot()) as string[];
  } catch {
    return [];
  }
}

export function awardAchievement(id: string) {
  const earned = getEarnedAchievements();
  if (earned.includes(id)) return;
  earned.push(id);
  localStorage.setItem(STORAGE_KEY, JSON.stringify(earned));
  window.dispatchEvent(new Event(CHANGE_EVENT));
}

export function checkAndAward(stats: {
  visited: number;
  cost: number;
  solver: string;
}) {
  for (const a of ACHIEVEMENTS) {
    if (a.check(stats) && !getEarnedAchievements().includes(a.id)) {
      awardAchievement(a.id);
    }
  }
}
