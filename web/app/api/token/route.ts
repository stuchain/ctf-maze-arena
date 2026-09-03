import { authOptions } from '@/auth';
import { getServerSession } from 'next-auth';
import { NextResponse } from 'next/server';
import jwt from 'jsonwebtoken';
import { serverEnv } from '@/lib/server-env';

const TOKEN_TTL_SECS = 10 * 60;

export async function GET() {
  if (serverEnv.AUTH_MODE === 'anonymous') {
    return NextResponse.json({ error: 'unauthorized' }, { status: 401 });
  }

  const session = await getServerSession(authOptions);
  if (!session?.user?.githubSubject) {
    return NextResponse.json({ error: 'unauthorized' }, { status: 401 });
  }

  const jwtSecret = serverEnv.JWT_SECRET;
  if (!jwtSecret) {
    return NextResponse.json({ error: 'jwt secret is not configured' }, { status: 500 });
  }

  const now = Math.floor(Date.now() / 1000);
  const payload = {
    sub: `github:${session.user.githubSubject}`,
    name: session.user.name ?? null,
    avatarUrl: session.user.image ?? null,
    iat: now,
    exp: now + TOKEN_TTL_SECS,
  };
  const token = jwt.sign(payload, jwtSecret, { algorithm: 'HS256' });

  return NextResponse.json({
    token,
    tokenType: 'Bearer',
    expiresAt: payload.exp,
  });
}
