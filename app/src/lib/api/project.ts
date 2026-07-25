import * as z from 'zod';

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

export async function updateProjectFile(
  projectId: string,
  fileId: string,
  text: string,
): Promise<UpdateFileResponse> {
  return UpdateFileResponseSchema.parse(
    await api.put(`project/${projectId}/file/${fileId}`, { json: { text } }).json(),
  );
}
