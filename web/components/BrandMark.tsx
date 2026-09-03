export function BrandMark({ size = 32 }: { size?: number }) {
  return (
    <svg
      viewBox="0 0 40 40"
      width={size}
      height={size}
      role="img"
      aria-label="CTF Maze Arena"
      className="brand-mark"
    >
      <rect x="1" y="1" width="38" height="38" rx="11" className="brand-mark__base" />
      <path d="M10 11h12v6h8v12H18v-6h-8z" className="brand-mark__maze" />
      <circle cx="11" cy="12" r="2.5" className="brand-mark__start" />
      <path d="m27 25 4 4m0-4-4 4" className="brand-mark__goal" />
    </svg>
  );
}

export function MazeGlyph({ size = 28 }: { size?: number }) {
  return (
    <svg viewBox="0 0 32 32" width={size} height={size} aria-hidden="true">
      <path d="M5 5h22v22H5zM10 5v7h7V9h5v8h5M5 17h8v10m4-5h5v5" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" />
      <circle cx="8" cy="8" r="2" fill="currentColor" />
    </svg>
  );
}
