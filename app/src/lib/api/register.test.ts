import { describe, expect, it } from 'vitest';

import { RegisterResponseSchema } from '@/lib/api/register';

describe('RegisterResponseSchema', () => {
  const validUser = {
    avatar_uri: null,
    created_at: '2024-01-01T00:00:00.000Z',
    id: 'user-1',
    nickname: 'Ada',
    updated_at: '2024-01-01T00:00:00.000Z',
    username: 'ada',
  };

  it('parses a valid register response', () => {
    const result = RegisterResponseSchema.safeParse({
      message: 'ok',
      payload: { token: 'abc.def.ghi', user: validUser },
    });

    expect(result.success).toBe(true);
  });

  it('rejects a response missing the payload', () => {
    const result = RegisterResponseSchema.safeParse({ message: 'ok' });

    expect(result.success).toBe(false);
  });
});
