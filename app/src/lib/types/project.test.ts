import { describe, expect, it } from 'vitest';

import { FileContentSchema, ProjectDetailSchema, ProjectSchema } from './project';

const validProject = {
  created_at: '2024-01-02T03:04:05Z',
  creator_id: 'u1',
  id: 'p1',
  name: 'Demo',
  owner_id: 'o1',
  owner_type: 'user',
  updated_at: '2024-01-02T03:04:05Z',
};

describe('ProjectSchema', () => {
  it('coerces date strings into Date instances', () => {
    const parsed = ProjectSchema.parse(validProject);
    expect(parsed.created_at).toBeInstanceOf(Date);
    expect(parsed.updated_at).toBeInstanceOf(Date);
    expect(parsed.owner_type).toBe('user');
  });

  it('rejects an unknown owner_type', () => {
    expect(() =>
      ProjectSchema.parse({ ...validProject, owner_type: 'bogus' }),
    ).toThrow();
  });
});

describe('FileContentSchema', () => {
  it('accepts a text file', () => {
    expect(FileContentSchema.parse({ kind: 'text', text: 'hi' })).toEqual({
      kind: 'text',
      text: 'hi',
    });
  });

  it('accepts a binary reference', () => {
    expect(
      FileContentSchema.parse({ kind: 'binary', storageKey: 'abc' }),
    ).toEqual({ kind: 'binary', storageKey: 'abc' });
  });

  it('rejects an unknown kind', () => {
    expect(() => FileContentSchema.parse({ kind: 'weird' })).toThrow();
  });
});

describe('ProjectDetailSchema', () => {
  it('parses a project with files and a null entry', () => {
    const detail = ProjectDetailSchema.parse({
      ...validProject,
      entry: null,
      files: [
        {
          content: { kind: 'text', text: 'x' },
          id: 'f1',
          path: 'main.typ',
          size: 1,
          updated_at: '2024-01-02T03:04:05Z',
          version: 0,
        },
      ],
    });
    expect(detail.entry).toBeNull();
    expect(detail.files[0].content.kind).toBe('text');
    expect(detail.files[0].updated_at).toBeInstanceOf(Date);
  });
});
