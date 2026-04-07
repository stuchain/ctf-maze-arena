import { authOptions } from "@/auth";
import { getServerSession } from "next-auth";
import { NextResponse } from "next/server";
import jwt from "jsonwebtoken";

const TOKEN_TTL_SECS = 10 * 60;

export async function GET() {
  let session = null;
  try {
    session = await getServerSession(authOptions);
  } catch {
    // If auth config is missing in environments like CI, treat as signed-out.
    session = null;
  }
  if (!session?.user) {
    return NextResponse.json({ error: "unauthorized" }, { status: 401 });
  }

  const jwtSecret = process.env.JWT_SECRET;
  if (!jwtSecret) {
    return NextResponse.json({ error: "jwt secret is not configured" }, { status: 500 });
  }

  const now = Math.floor(Date.now() / 1000);
  const payload = {
    sub: session.user.email ?? session.user.name ?? "unknown",
    name: session.user.name ?? null,
    iat: now,
    exp: now + TOKEN_TTL_SECS,
  };
  const token = jwt.sign(payload, jwtSecret, { algorithm: "HS256" });

  return NextResponse.json({
    token,
    tokenType: "Bearer",
    expiresAt: payload.exp,
  });
}
