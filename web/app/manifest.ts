import type { MetadataRoute } from 'next';

export default function manifest(): MetadataRoute.Manifest {
  return {
    name: 'CTF Maze Arena',
    short_name: 'Maze Arena',
    description: 'A deterministic pathfinding laboratory.',
    start_url: '/',
    display: 'standalone',
    background_color: '#070a10',
    theme_color: '#070a10',
    icons: [{ src: '/icon.svg', sizes: 'any', type: 'image/svg+xml' }],
  };
}
