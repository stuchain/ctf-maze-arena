'use client';

import Link from 'next/link';
import { signIn, signOut, useSession } from 'next-auth/react';
import { useEffect, useState } from 'react';
import { authenticatedHeaders } from '@/lib/auth-api';
import {
  deleteProfileResponseSchema,
  profileSchema,
  requestJson,
  type UserProfile,
} from '@/lib/api';
import { publicEnv } from '@/lib/env';
import { ConfirmDialog } from '@/components/ui/ConfirmDialog';
import { Button, Notice, Skeleton } from '@/components/ui/Primitives';

export function CommunityProfile({ refreshKey }: { refreshKey?: string | null }) {
  const { status } = useSession();
  const [profile, setProfile] = useState<UserProfile | null>(null);
  const [error, setError] = useState('');
  const [confirmDelete, setConfirmDelete] = useState(false);
  const [deleting, setDeleting] = useState(false);
  const authEnabled = publicEnv.NEXT_PUBLIC_AUTH_MODE !== 'anonymous';

  useEffect(() => {
    if (status !== 'authenticated') return;
    let active = true;
    authenticatedHeaders(false)
      .then((headers) => requestJson(`${publicEnv.NEXT_PUBLIC_API_URL}/api/profile`, profileSchema, { headers }))
      .then((data) => { if (active) setProfile(data); })
      .catch(() => { if (active) setError('Your profile could not be loaded. Try again shortly.'); });
    return () => { active = false; };
  }, [status, refreshKey]);

  const exportData = async () => {
    try {
      const headers = await authenticatedHeaders(false);
      const data = await requestJson(`${publicEnv.NEXT_PUBLIC_API_URL}/api/profile/export`, profileSchema, { headers });
      const url = URL.createObjectURL(new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' }));
      const anchor = document.createElement('a');
      anchor.href = url; anchor.download = 'ctf-maze-arena-data.json'; anchor.click();
      URL.revokeObjectURL(url);
    } catch { setError('Your data export could not be prepared.'); }
  };

  const deleteData = async () => {
    setDeleting(true); setError('');
    try {
      const headers = await authenticatedHeaders(false);
      await requestJson(`${publicEnv.NEXT_PUBLIC_API_URL}/api/profile`, deleteProfileResponseSchema, { method: 'DELETE', headers });
      setConfirmDelete(false);
      await signOut({ callbackUrl: '/' });
    } catch { setError('Your account data could not be deleted. Nothing was changed.'); }
    finally { setDeleting(false); }
  };

  if (status === 'loading' || (status === 'authenticated' && !profile && !error)) return <div className="profile-card" aria-label="Loading profile"><Skeleton /><Skeleton /></div>;
  if (status !== 'authenticated') return (
    <div className="profile-card">
      <strong>Keep your arena history</strong>
      <p>Anonymous play stays fully available. GitHub sign-in is only used for ranked submissions, streaks, durable achievements, and history.</p>
      {authEnabled ? <Button variant="secondary" size="sm" onClick={() => void signIn('github')}>Sign in with GitHub</Button> : <span className="community-hint">Persistent identity is disabled in this environment.</span>}
    </div>
  );

  return (
    <div className="profile-card">
      {error ? <Notice title="Profile action" tone="danger">{error}</Notice> : null}
      <div className="profile-summary">
        <div><strong>{profile?.displayName ?? 'GitHub player'}</strong><p>{profile?.totalSubmissions ?? 0} ranked runs · {profile?.dailyStreak ?? 0} day streak</p></div>
        <span>{profile?.achievements.length ?? 0} awards</span>
      </div>
      {profile?.history.length ? <ol className="profile-history" aria-label="Recent ranked runs">
        {profile.history.slice(0, 5).map((run) => <li key={run.runId}>
          <Link href={`/replay/${run.runId}`}>{run.solver} · cost {run.cost}</Link>
          <span>{run.challengeDate ? `Daily ${run.challengeDate}` : `${run.ms} ms`}</span>
        </li>)}
      </ol> : <p>No ranked history yet.</p>}
      <details className="privacy-copy"><summary>Privacy and stored data</summary><p>We store your GitHub subject, public name/avatar, owned runs, accepted scores, achievements, and daily streak facts. We never request repository access. Export is available at any time. Deletion removes profile and private progress; public ranked results remain anonymized as “Deleted player” to preserve leaderboard integrity.</p></details>
      <div className="profile-actions">
        <Button variant="ghost" size="sm" onClick={() => void exportData()}>Export my data</Button>
        <Button variant="destructive" size="sm" onClick={() => setConfirmDelete(true)}>Delete my data</Button>
      </div>
      <ConfirmDialog open={confirmDelete} title="Delete Your Arena Data?" description="This removes your profile, achievements, and private ownership. Public ranked entries are anonymized to preserve fair historical rankings." confirmLabel="Delete My Data" loading={deleting} onCancel={() => setConfirmDelete(false)} onConfirm={() => void deleteData()} />
    </div>
  );
}
