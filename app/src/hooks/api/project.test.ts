// @vitest-environment jsdom
import { act, renderHook } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import type { CreateProjectResponse } from '@/lib/api/project';

const post = vi.hoisted(() => vi.fn());

vi.mock('@/lib/request', () => ({ api: { post } }));

import { useCreateProject } from '@/hooks/api/project';

describe('useCreateProject', () => {
  beforeEach(() => {
    post.mockReset();
  });

  it('posts to the project endpoint and parses a valid response', async () => {
    post.mockReturnValue({
      json: () =>
        Promise.resolve({
          message: 'created',
          payload: {
            created_at: '2024-01-01T00:00:00.000Z',
            creator_id: 'user-1',
            id: 'project-1',
            name: 'Demo',
            owner_id: 'user-1',
            owner_type: 'user',
            updated_at: '2024-01-01T00:00:00.000Z',
          },
        }),
    });

    const { result } = renderHook(() => useCreateProject());

    const arg = { name: 'Demo', owner_id: 'user-1', owner_type: 'user' as const };
    let response: CreateProjectResponse | undefined;
    await act(async () => {
      response = await result.current.trigger(arg);
    });

    expect(post).toHaveBeenCalledWith('project', { json: arg });
    expect(response).toMatchObject({ message: 'created' });
    expect(response?.payload.created_at).toBeInstanceOf(Date);
  });

  it('rejects when the response payload fails schema validation', async () => {
    post.mockReturnValue({
      json: () => Promise.resolve({ message: 'created', payload: {} }),
    });

    const { result } = renderHook(() => useCreateProject());

    await expect(
      act(async () => {
        await result.current.trigger({
          name: 'Demo',
          owner_id: 'user-1',
          owner_type: 'user',
        });
      }),
    ).rejects.toThrow();
  });
});
