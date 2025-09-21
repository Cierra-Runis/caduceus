import z from 'zod';

import { ProjectPayload } from '../models/project';
import { ApiResponse } from '../response';

export const CreateProjectRequest = z.object({
  name: z
    .string('Project name is required')
    .nonempty('Project name is required'),
});

export type CreateProjectRequest = {
  owner_id: string;
  owner_type: 'team' | 'user';
} & z.infer<typeof CreateProjectRequest>;
export type CreateProjectResponse = ApiResponse<ProjectPayload>;
