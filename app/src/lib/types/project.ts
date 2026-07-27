import * as z from 'zod';

export type Project = z.infer<typeof ProjectSchema>;
export const ProjectSchema = z.object({
  created_at: z.string().trim().transform((str) => new Date(str)),
  creator_id: z.string().trim(),
  id: z.string().trim(),
  name: z.string().trim(),
  owner_id: z.string().trim(),
  owner_type: z.enum(['team', 'user']),
  // The pinned Typst version (a semver string) the server compiles this
  // project with; null means "follow the server default". Routes to a
  // tinymist worker binary server-side.
  pinned_version: z.string().trim().nullish(),
  updated_at: z.string().trim().transform((str) => new Date(str)),
});


// When the editor materializes a file's live text into a durable blob —
// mirrors VS Code's `files.autoSave`. Governs *blob materialization*, not
// durability (edits are always synced + snapshotted server-side).
export type AutoSavePolicy = z.infer<typeof AutoSavePolicySchema>;
export const AutoSavePolicySchema = z.enum([
  'afterDelay',
  'off',
  'onFocusChange',
  'onWindowChange',
]);

// Project-level editor settings, shared by every collaborator. Defaults match
// the server's, so a payload written before settings existed still parses.
export type ProjectSettings = z.infer<typeof ProjectSettingsSchema>;
export const ProjectSettingsSchema = z.object({
  autoSave: AutoSavePolicySchema.default('onFocusChange'),
  autoSaveDelay: z.number().default(1000),
});

// One node in the project's file `tree`, id-keyed on the wire. Structure +
// (for files) a blob reference — no inline text; text comes from the CRDT (the
// editor) or is fetched from the blob (download). Mirrors the server's
// `ProjectionEntry`.
export type TreeEntry = z.infer<typeof TreeEntrySchema>;
export const TreeEntrySchema = z.object({
  blob: z
    .object({ sha256: z.string().trim(), size: z.number() })
    .optional(),
  kind: z.enum(['file', 'folder']),
  name: z.string().trim(),
  parent: z.string().trim().nullish(),
  path: z.string().trim(),
});

// Editor-facing project ("open in editor"): carries the file `tree`
// (structure + blob refs, id-keyed) and the compile `entry` (a file id). Text
// is not inlined — the editor reads it from the CRDT; other consumers fetch the
// referenced blobs.
export type ProjectDetail = z.infer<typeof ProjectDetailSchema>;
export const ProjectDetailSchema = ProjectSchema.extend({
  entry: z.string().trim().nullable(),
  settings: ProjectSettingsSchema.default({
    autoSave: 'onFocusChange',
    autoSaveDelay: 1000,
  }),
  tree: z.record(z.string(), TreeEntrySchema).default({}),
});
