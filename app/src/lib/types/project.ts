import * as z from 'zod';

export type Project = z.infer<typeof ProjectSchema>;
export const ProjectSchema = z.object({
  created_at: z.string().trim().transform((str) => new Date(str)),
  creator_id: z.string().trim(),
  id: z.string().trim(),
  name: z.string().trim(),
  owner_id: z.string().trim(),
  owner_type: z.enum(['team', 'user']),
  updated_at: z.string().trim().transform((str) => new Date(str)),
});

// The content of a single file, as delivered to the editor. Text is inlined so
// the compiler can use it immediately; a binary is only a reference until asset
// delivery lands (M3).
export type FileContent = z.infer<typeof FileContentSchema>;
export const FileContentSchema = z.discriminatedUnion('kind', [
  z.object({ kind: z.literal('text'), text: z.string().trim() }),
  z.object({ kind: z.literal('binary'), storageKey: z.string().trim() }),
]);

export type ProjectFile = z.infer<typeof ProjectFileSchema>;
export const ProjectFileSchema = z.object({
  content: FileContentSchema,
  id: z.string().trim(),
  path: z.string().trim(),
  size: z.number(),
  updated_at: z.string().trim().transform((str) => new Date(str)),
  version: z.number(),
});

// Editor-facing project ("open in editor"): carries the whole virtual file
// system with text content inlined, plus the compile `entry`. `entry` is a file
// id (resolved to a path against `files`); null for a project with no entry.
export type ProjectDetail = z.infer<typeof ProjectDetailSchema>;
export const ProjectDetailSchema = ProjectSchema.extend({
  entry: z.string().trim().nullable(),
  files: z.array(ProjectFileSchema),
});
