import { describe, expect, it } from 'vitest';

import { UserProjectResponseSchema } from '@/lib/api/user/project';

describe('UserProjectResponseSchema', () => {
  const project = {
    created_at: '2024-01-01T00:00:00.000Z',
    creator_id: 'user-1',
    id: 'project-1',
    name: 'Untitled',
    owner_id: 'user-1',
    owner_type: 'user',
    updated_at: '2024-01-01T00:00:00.000Z',
  };

  it('parses a list of projects', () => {
    const result = UserProjectResponseSchema.parse({ payload: [project] });

    expect(result.payload[0]?.created_at).toBeInstanceOf(Date);
  });

  it('rejects a payload item missing required fields', () => {
    const result = UserProjectResponseSchema.safeParse({
      payload: [{ id: 'project-1' }],
    });

    expect(result.success).toBe(false);
  });
});
