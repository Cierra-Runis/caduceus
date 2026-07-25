import { describe, expect, it } from 'vitest';

import { TeamSchema } from '@/lib/types/team';

describe('TeamSchema', () => {
  const valid = {
    avatar_uri: 'https://example.com/avatar.png',
    id: 'team-1',
    name: ' Acme ',
  };

  it('parses a valid team and trims the name', () => {
    const result = TeamSchema.parse(valid);

    expect(result.name).toBe('Acme');
  });

  it('accepts a null avatar_uri', () => {
    const result = TeamSchema.safeParse({ ...valid, avatar_uri: null });

    expect(result.success).toBe(true);
  });

  it('rejects a missing id', () => {
    const result = TeamSchema.safeParse({
      avatar_uri: null,
      name: 'Acme',
    });

    expect(result.success).toBe(false);
  });
});
