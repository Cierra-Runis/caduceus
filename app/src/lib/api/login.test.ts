import { describe, expect, it } from 'vitest';

import { LoginResponseSchema } from '@/lib/api/login';

describe('LoginResponseSchema', () => {
  const validUser = {
    avatar_uri: null,
    created_at: '2024-01-01T00:00:00.000Z',
    id: 'user-1',
    nickname: 'Ada',
    updated_at: '2024-01-01T00:00:00.000Z',
    username: 'ada',
  };

  it('parses a valid login response and transforms nested user dates', () => {
    const result = LoginResponseSchema.parse({
      message: 'ok',
      payload: { token: ' abc.def.ghi ', user: validUser },
    });

    expect(result.payload.token).toBe('abc.def.ghi');
    expect(result.payload.user.created_at).toBeInstanceOf(Date);
  });

  it('rejects a response missing the token', () => {
    const result = LoginResponseSchema.safeParse({
      message: 'ok',
      payload: { user: validUser },
    });

    expect(result.success).toBe(false);
  });

  it('rejects a response with a malformed user', () => {
    const result = LoginResponseSchema.safeParse({
      message: 'ok',
      payload: { token: 'abc', user: { ...validUser, username: undefined } },
    });

    expect(result.success).toBe(false);
  });
});
