import 'server-only';

import { jwtVerify } from 'jose';

const secretKey = process.env.JWT_SECRET;
const encodedKey = new TextEncoder().encode(secretKey);

export async function decrypt(session: string | undefined = '') {
  try {
    const { payload } = await jwtVerify(session, encodedKey, {
      algorithms: ['HS256'],
    });
    return payload;
  } catch (error) {
    console.error('Failed to verify session:', error);
  }
}
