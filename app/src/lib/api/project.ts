import { HTTPError } from 'ky';
import * as z from 'zod';

import { env } from '@/lib/env';
import { api } from '@/lib/request';
import { ProjectDetailSchema, ProjectSchema } from '@/lib/types/project';

export type CreateProjectRequest = {
  owner_id: string;
  owner_type: 'team' | 'user';
} & z.infer<typeof CreateProjectRequestSchema>;
export const CreateProjectRequestSchema = z.object({
  name: z
    .string('Project name is required').trim()
    .nonempty('Project name is required'),
});

export type CreateProjectResponse = z.infer<typeof CreateProjectResponseSchema>;
export const CreateProjectResponseSchema = z.object({
  message: z.string().trim(),
  payload: ProjectSchema,
});

// Response for cloning a project (POST /project/{id}/duplicate): the new
// project, owned the same way as the source, with the requester as creator.
export type DuplicateProjectResponse = z.infer<
  typeof DuplicateProjectResponseSchema
>;
export const DuplicateProjectResponseSchema = z.object({
  message: z.string().trim(),
  payload: ProjectSchema,
});

// Response for opening a single project in the editor (GET /project/{id}):
// the detail payload with the full file tree and inlined content.
export type ProjectDetailResponse = z.infer<typeof ProjectDetailResponseSchema>;
export const ProjectDetailResponseSchema = z.object({
  message: z.string().trim(),
  payload: ProjectDetailSchema,
});

export type UpdateProjectRequest = z.infer<typeof UpdateProjectRequestSchema>;
export const UpdateProjectRequestSchema = z.object({
  name: z.string().trim().nonempty('Project name is required'),
  owner_id: z.string().trim(),
  owner_type: z.enum(['team', 'user']),
});

// Response for updating a project's metadata (PUT /project/{id}): the project
// with its new name/owner applied.
export type UpdateProjectResponse = z.infer<typeof UpdateProjectResponseSchema>;
export const UpdateProjectResponseSchema = z.object({
  message: z.string().trim(),
  payload: ProjectSchema,
});

// Persist a single file's text content (whole-buffer save). Returns the file's
// freshly bumped version/timestamp.
export type UpdateFileResponse = z.infer<typeof UpdateFileResponseSchema>;
export const UpdateFileResponseSchema = z.object({
  message: z.string().trim(),
  payload: z.object({
    id: z.string().trim(),
    updated_at: z.string().trim().transform((str) => new Date(str)),
    version: z.number(),
  }),
});

/** Best-effort human message from a failed API call, for toasts. */
export async function apiErrorMessage(error: unknown): Promise<string> {
  if (error instanceof HTTPError) {
    try {
      const body = (await error.response.json()) as { message?: string };
      if (body?.message) return body.message;
    } catch {
      // fall through to the generic message
    }
  }
  return error instanceof Error ? error.message : String(error);
}

// --- File-tree structural operations ---------------------------------------
//
// Each mutation returns nothing useful to the caller: the file explorer holds
// the project detail in SWR and revalidates it after any successful change, so
// the tree always reflects the server's authoritative view (paths, ids,
// entry) rather than a locally-guessed one.

/** Create a new empty text file at `path` (relative to the project root). */
export async function createFile(projectId: string, path: string): Promise<void> {
  await api.post(`project/${projectId}/file`, { json: { path } });
}

/** Create a new empty folder at `path`. */
export async function createFolder(projectId: string, path: string): Promise<void> {
  await api.post(`project/${projectId}/folder`, { json: { path } });
}

/** Delete a single file. */
export async function deleteFile(projectId: string, fileId: string): Promise<void> {
  await api.delete(`project/${projectId}/file/${fileId}`);
}

/** Delete a folder and everything under it. */
export async function deleteFolder(projectId: string, path: string): Promise<void> {
  await api.delete(`project/${projectId}/folder`, { json: { path } });
}

/** Fetch the full project detail (used to revalidate the tree after a change). */
export async function fetchProjectDetail(projectId: string): Promise<ProjectDetailResponse> {
  return ProjectDetailResponseSchema.parse(await api.get(`project/${projectId}`).json());
}

/** Absolute URL for a binary asset's raw bytes (for previews / downloads). */
export function fileRawUrl(projectId: string, fileId: string): string {
  return `${env.NEXT_PUBLIC_API_URL}/project/${projectId}/file/${fileId}/raw`;
}

/** Rename or move a single file to `path`. */
export async function renameFile(
  projectId: string,
  fileId: string,
  path: string,
): Promise<void> {
  await api.patch(`project/${projectId}/file/${fileId}`, { json: { path } });
}

/** Set the project's compile entry to an existing file. */
export async function setEntry(projectId: string, fileId: string): Promise<void> {
  await api.post(`project/${projectId}/entry`, { json: { file_id: fileId } });
}

export async function updateProjectFile(
  projectId: string,
  fileId: string,
  text: string,
): Promise<UpdateFileResponse> {
  return UpdateFileResponseSchema.parse(
    await api.put(`project/${projectId}/file/${fileId}`, { json: { text } }).json(),
  );
}

/**
 * Upload one or more binary assets. The multipart body must name each file
 * part after its target path relative to the project root (see the upload
 * dialog): `form.append(targetPath, file)`.
 */
export async function uploadFiles(projectId: string, form: FormData): Promise<void> {
  await api.post(`project/${projectId}/upload`, { body: form });
}
