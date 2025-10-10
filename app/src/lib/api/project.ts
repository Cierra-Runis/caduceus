import * as z from 'zod';

import { ProjectSchema } from '@/lib/types/project';

export type CreateProjectRequest = {
  owner_id: string;
  owner_type: 'team' | 'user';
} & z.infer<typeof CreateProjectRequestSchema>;
export const CreateProjectRequestSchema = z.object({
  name: z
    .string('Project name is required')
    .nonempty('Project name is required'),
});

export type CreateProjectResponse = z.infer<typeof CreateProjectResponseSchema>;
export const CreateProjectResponseSchema = z.object({
  message: z.string(),
  payload: ProjectSchema,
});

export type UpdateProjectRequest = z.infer<typeof UpdateProjectRequestSchema>;
export const UpdateProjectRequestSchema = z.object({
  name: z.string().nonempty('Project name is required'),
  owner_id: z.string(),
  owner_type: z.enum(['team', 'user']),
});
