import { MazeGrid } from '../components/MazeGrid';

export default function Home() {
  return (
    <div className="flex min-h-screen items-center justify-center bg-zinc-50 font-sans dark:bg-black">
      <div className="p-8">
        <MazeGrid width={10} height={10} />
      </div>
    </div>
  );
}
