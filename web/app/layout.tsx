import type { Metadata, Viewport } from "next";
import { Geist, Geist_Mono } from "next/font/google";
import Script from "next/script";
import { AuthSessionProvider } from "@/components/AuthSessionProvider";
import "./globals.css";

const geistSans = Geist({
  variable: "--font-geist-sans",
  subsets: ["latin"],
});

const geistMono = Geist_Mono({
  variable: "--font-geist-mono",
  subsets: ["latin"],
});

const metadataOrigin = process.env.NEXT_PUBLIC_SITE_URL
  ?? (process.env.VERCEL_PROJECT_PRODUCTION_URL
    ? `https://${process.env.VERCEL_PROJECT_PRODUCTION_URL}`
    : 'http://localhost:3000');

export const metadata: Metadata = {
  metadataBase: new URL(metadataOrigin),
  title: {
    default: 'CTF Maze Arena',
    template: '%s | CTF Maze Arena',
  },
  description: 'Generate deterministic mazes, visualize pathfinding algorithms, and compare solver performance.',
  applicationName: 'CTF Maze Arena',
  alternates: { canonical: '/' },
  authors: [{ name: 'CTF Maze Arena contributors' }],
  creator: 'CTF Maze Arena',
  keywords: ['pathfinding', 'maze', 'algorithms', 'visualizer', 'CTF'],
  openGraph: {
    type: 'website',
    title: 'CTF Maze Arena',
    description: 'A deterministic pathfinding laboratory for exploring and comparing graph-search algorithms.',
    siteName: 'CTF Maze Arena',
  },
  twitter: {
    card: 'summary_large_image',
    title: 'CTF Maze Arena',
    description: 'Generate a maze. Run a solver. Inspect every decision.',
  },
};

export const viewport: Viewport = {
  colorScheme: 'dark light',
  themeColor: [
    { media: '(prefers-color-scheme: dark)', color: '#070a10' },
    { media: '(prefers-color-scheme: light)', color: '#f4f6f9' },
  ],
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en" suppressHydrationWarning>
      <body
        className={`${geistSans.variable} ${geistMono.variable} antialiased`}
      >
        <a href="#main-content" className="skip-link">
          Skip to main content
        </a>
        <Script id="theme-init" strategy="beforeInteractive">{`
          try {
            var storedTheme = localStorage.getItem('ctf-maze-theme');
            var preferredTheme = window.matchMedia('(prefers-color-scheme: light)').matches ? 'light' : 'dark';
            var theme = storedTheme === 'light' || storedTheme === 'dark' ? storedTheme : preferredTheme;
            document.documentElement.dataset.theme = theme;
            document.documentElement.style.colorScheme = theme;
          } catch (_) {
            document.documentElement.dataset.theme = 'dark';
          }
        `}</Script>
        <AuthSessionProvider>{children}</AuthSessionProvider>
      </body>
    </html>
  );
}
