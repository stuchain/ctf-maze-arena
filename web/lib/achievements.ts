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

export function getEarnedAchievements(): string[] {
  if (typeof window === 'undefined') return [];
  try {
    return JSON.parse(localStorage.getItem(STORAGE_KEY) || '[]') as string[];
  } catch {
    return [];
  }
}

export function awardAchievement(id: string) {
  const earned = getEarnedAchievements();
  if (earned.includes(id)) return;
  earned.push(id);
  localStorage.setItem(STORAGE_KEY, JSON.stringify(earned));
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
