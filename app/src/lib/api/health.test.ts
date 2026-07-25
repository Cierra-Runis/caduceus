import { describe, expect, it } from 'vitest';

import { HealthResponseSchema } from '@/lib/api/health';

describe('HealthResponseSchema', () => {
  it('parses a healthy response', () => {
    const result = HealthResponseSchema.safeParse({
      message: 'ok',
      payload: { status: 'healthy' },
    });

    expect(result.success).toBe(true);
  });

  it('rejects a status outside the enum', () => {
    const result = HealthResponseSchema.safeParse({
      message: 'ok',
      payload: { status: 'degraded' },
    });

    expect(result.success).toBe(false);
  });
});
