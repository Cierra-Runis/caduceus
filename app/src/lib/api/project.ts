import useSWRMutation from 'swr/mutation';
import * as z from 'zod';

import { Project } from '@/lib/types/project';

import { api } from '../request';

export type CreateProjectRequest = {
  owner_id: string;
  owner_type: 'team' | 'user';
} & z.infer<typeof CreateProjectRequest>;
export const CreateProjectRequest = z.object({
  name: z
    .string('Project name is required')
    .nonempty('Project name is required'),
});

export type CreateProjectResponse = z.infer<typeof CreateProjectResponse>;
export const CreateProjectResponse = z.object({
  message: z.string(),
  payload: Project,
});

export const useCreateProject = () => {
  return useSWRMutation<
    CreateProjectResponse,
    Error,
    string,
    CreateProjectRequest
  >('project', (key, { arg }) => api.post(key, { json: arg }).json());
};
