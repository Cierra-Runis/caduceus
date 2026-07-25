import { describe, expect, it } from 'vitest';

import {
  FileContentSchema,
  ProjectDetailSchema,
  ProjectFileSchema,
  ProjectSchema,
} from '@/lib/types/project';

describe('ProjectSchema', () => {
  const valid = {
    created_at: '2024-01-01T00:00:00.000Z',
    creator_id: 'user-1',
    id: 'project-1',
    name: ' Untitled ',
    owner_id: 'user-1',
    owner_type: 'user',
    updated_at: '2024-01-02T00:00:00.000Z',
  };

  it('parses a valid project, transforming dates and trimming the name', () => {
    const result = ProjectSchema.parse(valid);

    expect(result.created_at).toBeInstanceOf(Date);
    expect(result.updated_at).toBeInstanceOf(Date);
    expect(result.created_at.toISOString()).toBe('2024-01-01T00:00:00.000Z');
    expect(result.name).toBe('Untitled');
  });

  it('accepts owner_type "team"', () => {
    const result = ProjectSchema.safeParse({ ...valid, owner_type: 'team' });

    expect(result.success).toBe(true);
  });

  it('rejects an owner_type outside the enum', () => {
    const result = ProjectSchema.safeParse({ ...valid, owner_type: 'org' });

    expect(result.success).toBe(false);
  });

  it('rejects a missing id', () => {
    const result = ProjectSchema.safeParse({
      created_at: '2024-01-01T00:00:00.000Z',
      creator_id: 'user-1',
      name: 'Untitled',
      owner_id: 'user-1',
      owner_type: 'user',
      updated_at: '2024-01-02T00:00:00.000Z',
    });

    expect(result.success).toBe(false);
  });
});

describe('FileContentSchema', () => {
  it('parses a text file', () => {
    const result = FileContentSchema.safeParse({ kind: 'text', text: 'hello' });

    expect(result.success).toBe(true);
  });

  it('parses a binary file', () => {
    const result = FileContentSchema.safeParse({
      kind: 'binary',
      storageKey: 'blob-1',
    });

    expect(result.success).toBe(true);
  });

  it('rejects a kind outside the union', () => {
    const result = FileContentSchema.safeParse({ kind: 'video', text: 'nope' });

    expect(result.success).toBe(false);
  });

  it('rejects a binary variant missing storageKey', () => {
    const result = FileContentSchema.safeParse({ kind: 'binary' });

    expect(result.success).toBe(false);
  });
});

describe('ProjectFileSchema', () => {
  const valid = {
    content: { kind: 'text', text: 'hi' },
    id: 'file-1',
    path: 'main.typ',
    size: 2,
    updated_at: '2024-01-01T00:00:00.000Z',
    version: 1,
  };

  it('parses a valid file, transforming updated_at', () => {
    const result = ProjectFileSchema.parse(valid);

    expect(result.updated_at).toBeInstanceOf(Date);
  });

  it('rejects a non-numeric size', () => {
    const result = ProjectFileSchema.safeParse({ ...valid, size: '2' });

    expect(result.success).toBe(false);
  });
});

describe('ProjectDetailSchema', () => {
  const valid = {
    created_at: '2024-01-01T00:00:00.000Z',
    creator_id: 'user-1',
    entry: 'main.typ',
    files: [],
    id: 'project-1',
    name: 'Untitled',
    owner_id: 'user-1',
    owner_type: 'team',
    updated_at: '2024-01-01T00:00:00.000Z',
  };

  it('parses a project extended with entry and files', () => {
    const result = ProjectDetailSchema.safeParse(valid);

    expect(result.success).toBe(true);
  });

  it('allows a null entry', () => {
    const result = ProjectDetailSchema.safeParse({ ...valid, entry: null });

    expect(result.success).toBe(true);
  });

  it('rejects a missing files array', () => {
    const result = ProjectDetailSchema.safeParse({
      created_at: '2024-01-01T00:00:00.000Z',
      creator_id: 'user-1',
      entry: null,
      id: 'project-1',
      name: 'Untitled',
      owner_id: 'user-1',
      owner_type: 'team',
      updated_at: '2024-01-01T00:00:00.000Z',
    });

    expect(result.success).toBe(false);
  });
});
