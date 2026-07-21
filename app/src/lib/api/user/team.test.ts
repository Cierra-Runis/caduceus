import { describe, expect, it } from 'vitest';

import { RouteUserTeamsSchema } from '@/lib/api/user/team';

describe('RouteUserTeamsSchema', () => {
  it('parses a list of teams', () => {
    const result = RouteUserTeamsSchema.safeParse({
      message: 'ok',
      payload: [{ avatar_uri: null, id: 'team-1', name: 'Acme' }],
    });

    expect(result.success).toBe(true);
  });

  it('rejects a non-array payload', () => {
    const result = RouteUserTeamsSchema.safeParse({
      message: 'ok',
      payload: { avatar_uri: null, id: 'team-1', name: 'Acme' },
    });

    expect(result.success).toBe(false);
  });
});
