import * as z from 'zod';

import { ProjectSchema } from '@/lib/types/project';
import { TeamSchema } from '@/lib/types/team';

export const CreateTeamRequestSchema = z.object({
  name: z.string('Team name is required').nonempty('Team name is required'),
});
export type CreateTeamRequest = z.infer<typeof CreateTeamRequestSchema>;

export const CreateTeamResponseSchema = z.object({
  message: z.string(),
  payload: TeamSchema,
});
export type CreateTeamResponse = z.infer<typeof CreateTeamResponseSchema>;

export type TeamProjectResponse = z.infer<typeof TeamProjectResponseSchema>;
export const TeamProjectResponseSchema = z.object({
  payload: z.array(ProjectSchema),
});
