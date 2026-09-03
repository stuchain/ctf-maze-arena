import { ImageResponse } from 'next/og';

export const alt = 'CTF Maze Arena — deterministic pathfinding laboratory';
export const size = { width: 1200, height: 630 };
export const contentType = 'image/png';

export default function OpenGraphImage() {
  return new ImageResponse(
    <div
      style={{
        width: '100%', height: '100%', display: 'flex', flexDirection: 'column',
        justifyContent: 'space-between', padding: 72, color: '#f7fafc',
        background: 'linear-gradient(135deg, #070a10 0%, #101827 62%, #09251f 100%)',
        fontFamily: 'sans-serif',
      }}
    >
      <div style={{ display: 'flex', alignItems: 'center', gap: 20, color: '#6ee7b7', fontSize: 30, fontWeight: 700 }}>
        <span style={{ display: 'flex', width: 48, height: 48, border: '3px solid #6ee7b7', borderRadius: 12 }} />
        CTF MAZE ARENA
      </div>
      <div style={{ display: 'flex', flexDirection: 'column', gap: 24 }}>
        <div style={{ display: 'flex', maxWidth: 920, fontSize: 76, fontWeight: 800, lineHeight: 1.02 }}>
          See how search finds a way through.
        </div>
        <div style={{ display: 'flex', color: '#9ca9b8', fontSize: 30 }}>
          Generate a maze · Run a solver · Inspect every decision
        </div>
      </div>
      <div style={{ display: 'flex', gap: 24, color: '#6ee7b7', fontSize: 22, fontFamily: 'monospace' }}>
        BFS <span style={{ color: '#465366' }}>·</span> DIJKSTRA <span style={{ color: '#465366' }}>·</span> A*
      </div>
    </div>,
    size,
  );
}
