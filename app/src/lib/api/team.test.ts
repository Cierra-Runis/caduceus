import { describe, expect, it } from 'vitest';

import {
  CreateTeamRequestSchema,
  CreateTeamResponseSchema,
  TeamProjectResponseSchema,
} from '@/lib/api/team';

describe('CreateTeamRequestSchema', () => {
  it('parses and trims a valid team name', () => {
    const result = CreateTeamRequestSchema.parse({ name: ' Acme ' });

    expect(result.name).toBe('Acme');
  });

  it('rejects an empty name', () => {
    const result = CreateTeamRequestSchema.safeParse({ name: '   ' });

    expect(result.success).toBe(false);
  });
});

describe('CreateTeamResponseSchema', () => {
  it('parses a valid response', () => {
    const result = CreateTeamResponseSchema.safeParse({
      message: 'ok',
      payload: { avatar_uri: null, id: 'team-1', name: 'Acme' },
    });

    expect(result.success).toBe(true);
  });

  it('rejects a response with a malformed team payload', () => {
    const result = CreateTeamResponseSchema.safeParse({
      message: 'ok',
      payload: { avatar_uri: null, name: 'Acme' },
    });

    expect(result.success).toBe(false);
  });
});

describe('TeamProjectResponseSchema', () => {
  const project = {
    created_at: '2024-01-01T00:00:00.000Z',
    creator_id: 'user-1',
    id: 'project-1',
    name: 'Untitled',
    owner_id: 'team-1',
    owner_type: 'team',
    updated_at: '2024-01-01T00:00:00.000Z',
  };

  it('parses a list of projects', () => {
    const result = TeamProjectResponseSchema.parse({ payload: [project] });

    expect(result.payload).toHaveLength(1);
    expect(result.payload[0]?.created_at).toBeInstanceOf(Date);
  });

  it('rejects a non-array payload', () => {
    const result = TeamProjectResponseSchema.safeParse({ payload: project });

    expect(result.success).toBe(false);
  });
});
