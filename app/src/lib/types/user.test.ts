import { describe, expect, it } from 'vitest';

import { UserSchema } from '@/lib/types/user';

describe('UserSchema', () => {
  const valid = {
    avatar_uri: null,
    created_at: '2024-01-01T00:00:00.000Z',
    id: 'user-1',
    nickname: ' Ada ',
    updated_at: '2024-01-02T00:00:00.000Z',
    username: 'ada',
  };

  it('parses a valid user, transforming dates and trimming strings', () => {
    const result = UserSchema.parse(valid);

    expect(result.created_at).toBeInstanceOf(Date);
    expect(result.created_at.toISOString()).toBe('2024-01-01T00:00:00.000Z');
    expect(result.nickname).toBe('Ada');
  });

  it('accepts a null avatar_uri', () => {
    const result = UserSchema.safeParse(valid);

    expect(result.success).toBe(true);
  });

  it('rejects a missing username', () => {
    const result = UserSchema.safeParse({
      avatar_uri: null,
      created_at: '2024-01-01T00:00:00.000Z',
      id: 'user-1',
      nickname: 'Ada',
      updated_at: '2024-01-02T00:00:00.000Z',
    });

    expect(result.success).toBe(false);
  });

  it('rejects a non-string avatar_uri', () => {
    const result = UserSchema.safeParse({ ...valid, avatar_uri: 42 });

    expect(result.success).toBe(false);
  });
});
