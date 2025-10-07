import useSWRMutation from 'swr/mutation';
import * as z from 'zod';

import { ProjectSchema } from '@/lib/types/project';

import { api } from '../request';

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

export const useCreateProject = () => {
  return useSWRMutation<
    CreateProjectResponse,
    Error,
    string,
    CreateProjectRequest
  >('project', (key, { arg }) => api.post(key, { json: arg }).json());
};
