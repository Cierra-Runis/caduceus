import { describe, expect, it } from 'vitest';

import { RouteUserMeSchema } from '@/lib/api/user/me';

describe('RouteUserMeSchema', () => {
  const user = {
    avatar_uri: null,
    created_at: '2024-01-01T00:00:00.000Z',
    id: 'user-1',
    nickname: 'Ada',
    updated_at: '2024-01-01T00:00:00.000Z',
    username: 'ada',
  };

  it('parses a valid response', () => {
    const result = RouteUserMeSchema.safeParse({ message: 'ok', payload: user });

    expect(result.success).toBe(true);
  });

  it('rejects a response with a malformed user payload', () => {
    const result = RouteUserMeSchema.safeParse({
      message: 'ok',
      payload: { ...user, id: undefined },
    });

    expect(result.success).toBe(false);
  });
});
