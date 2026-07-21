import { describe, expect, it } from 'vitest';

import {
  CreateProjectRequestSchema,
  CreateProjectResponseSchema,
  ProjectDetailResponseSchema,
  UpdateFileResponseSchema,
  UpdateProjectRequestSchema,
} from '@/lib/api/project';

const project = {
  created_at: '2024-01-01T00:00:00.000Z',
  creator_id: 'user-1',
  id: 'project-1',
  name: 'Untitled',
  owner_id: 'user-1',
  owner_type: 'user',
  updated_at: '2024-01-01T00:00:00.000Z',
};

describe('CreateProjectRequestSchema', () => {
  it('parses and trims a valid name', () => {
    const result = CreateProjectRequestSchema.parse({ name: ' Untitled ' });

    expect(result.name).toBe('Untitled');
  });

  it('rejects an empty name', () => {
    const result = CreateProjectRequestSchema.safeParse({ name: '   ' });

    expect(result.success).toBe(false);
  });
});

describe('CreateProjectResponseSchema', () => {
  it('parses a valid response', () => {
    const result = CreateProjectResponseSchema.safeParse({
      message: 'ok',
      payload: project,
    });

    expect(result.success).toBe(true);
  });

  it('rejects a response with a malformed project payload', () => {
    const result = CreateProjectResponseSchema.safeParse({
      message: 'ok',
      payload: { ...project, owner_type: 'org' },
    });

    expect(result.success).toBe(false);
  });
});

describe('ProjectDetailResponseSchema', () => {
  it('parses a valid detail response', () => {
    const result = ProjectDetailResponseSchema.safeParse({
      message: 'ok',
      payload: { ...project, entry: null, files: [] },
    });

    expect(result.success).toBe(true);
  });

  it('rejects a detail response missing files', () => {
    const result = ProjectDetailResponseSchema.safeParse({
      message: 'ok',
      payload: { ...project, entry: null },
    });

    expect(result.success).toBe(false);
  });
});

describe('UpdateProjectRequestSchema', () => {
  it('parses and trims a valid request', () => {
    const result = UpdateProjectRequestSchema.parse({
      name: ' Untitled ',
      owner_id: ' user-1 ',
      owner_type: 'user',
    });

    expect(result).toEqual({
      name: 'Untitled',
      owner_id: 'user-1',
      owner_type: 'user',
    });
  });

  it('rejects an owner_type outside the enum', () => {
    const result = UpdateProjectRequestSchema.safeParse({
      name: 'Untitled',
      owner_id: 'user-1',
      owner_type: 'org',
    });

    expect(result.success).toBe(false);
  });
});

describe('UpdateFileResponseSchema', () => {
  it('parses a valid response and transforms updated_at', () => {
    const result = UpdateFileResponseSchema.parse({
      message: 'ok',
      payload: {
        id: 'file-1',
        updated_at: '2024-01-01T00:00:00.000Z',
        version: 2,
      },
    });

    expect(result.payload.updated_at).toBeInstanceOf(Date);
  });

  it('rejects a non-numeric version', () => {
    const result = UpdateFileResponseSchema.safeParse({
      message: 'ok',
      payload: {
        id: 'file-1',
        updated_at: '2024-01-01T00:00:00.000Z',
        version: '2',
      },
    });

    expect(result.success).toBe(false);
  });
});
