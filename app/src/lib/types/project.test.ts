import { describe, expect, it } from 'vitest';

import { ProjectDetailSchema, ProjectSchema } from './project';

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

  it('carries pinned_version when present and defaults it to nullish', () => {
    expect(ProjectSchema.parse(validProject).pinned_version).toBeUndefined();
    expect(
      ProjectSchema.parse({ ...validProject, pinned_version: '0.13.1' })
        .pinned_version,
    ).toBe('0.13.1');
  });
});

describe('ProjectDetailSchema', () => {
  it('parses a project with a tree and a null entry', () => {
    const detail = ProjectDetailSchema.parse({
      ...validProject,
      entry: null,
      tree: {
        f1: {
          blob: { sha256: 'a'.repeat(64), size: 1 },
          kind: 'file',
          name: 'main.typ',
          path: 'main.typ',
        },
      },
    });
    expect(detail.entry).toBeNull();
    expect(detail.tree.f1.kind).toBe('file');
    expect(detail.tree.f1.blob?.sha256).toBe('a'.repeat(64));
  });

  it('defaults settings when absent', () => {
    const detail = ProjectDetailSchema.parse({
      ...validProject,
      entry: null,
      tree: {},
    });
    expect(detail.settings.autoSave).toBe('onFocusChange');
  });
});
