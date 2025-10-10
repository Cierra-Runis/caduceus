import { JWTPayload, jwtVerify } from 'jose';
import { cookies } from 'next/headers';
import { redirect } from 'next/navigation';

import { env } from '@/lib/env';
import 'server-only';

const encodedKey = new TextEncoder().encode(env.JWT_SECRET);

export async function decrypt(
  token: string | undefined,
): Promise<JWTPayload | undefined> {
  if (!token) return;
  try {
    const { payload } = await jwtVerify(token, encodedKey, {
      algorithms: ['HS512'],
    });
    return payload;
  } catch (error) {
    console.error('[JWTPayload decrypt] Failed to verify session:', error);
    return;
  }
}

export async function deleteJwt() {
  const cookie = await cookies();
  cookie.delete('token');
  redirect('/');
}
