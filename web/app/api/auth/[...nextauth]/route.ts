import NextAuth from 'next-auth';
import { authOptions } from '@/auth';
import { NextResponse } from 'next/server';
import { serverEnv } from '@/lib/server-env';

const handler = NextAuth(authOptions);
const unavailable = () =>
  NextResponse.json({ error: 'authentication is disabled' }, { status: 404 });
const anonymousGet = (request: Request) => request.url.endsWith('/api/auth/session')
  ? NextResponse.json({})
  : unavailable();

export const GET = serverEnv.AUTH_MODE === 'anonymous' ? anonymousGet : handler;
export const POST = serverEnv.AUTH_MODE === 'anonymous' ? unavailable : handler;
