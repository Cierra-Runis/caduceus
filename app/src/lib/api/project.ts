import * as z from 'zod';

import { api } from '@/lib/request';
import {
  ProjectDetailSchema,
  ProjectSchema,
  ProjectSettings,
  ProjectSettingsSchema,
} from '@/lib/types/project';

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

// Response for updating a project's editor settings (PUT /project/{id}/settings).
export const UpdateSettingsResponseSchema = z.object({
  message: z.string().trim(),
  payload: ProjectSettingsSchema,
});

// Ask the server to materialize the room's live text into blobs now — the
// client's `files.autoSave` policy fired. Fire-and-forget; errors are ignored
// (the periodic snapshot still holds the text durably).
export async function flushProject(projectId: string): Promise<void> {
  await api.post(`project/${projectId}/flush`).json();
}

// Persist the project's editor settings (auto-save policy, …). Shared by every
// collaborator; returns the stored settings.
export async function updateProjectSettings(
  projectId: string,
  settings: ProjectSettings,
): Promise<ProjectSettings> {
  const res = UpdateSettingsResponseSchema.parse(
    await api.put(`project/${projectId}/settings`, { json: settings }).json(),
  );
  return res.payload;
}
