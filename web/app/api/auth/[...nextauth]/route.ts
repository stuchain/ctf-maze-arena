import NextAuth from 'next-auth';
import { authOptions } from '@/auth';
import { NextResponse } from 'next/server';
import { serverEnv } from '@/lib/server-env';

const handler = NextAuth(authOptions);
const unavailable = () =>
  NextResponse.json({ error: 'authentication is disabled' }, { status: 404 });

export const GET = serverEnv.AUTH_MODE === 'anonymous' ? unavailable : handler;
export const POST = serverEnv.AUTH_MODE === 'anonymous' ? unavailable : handler;
