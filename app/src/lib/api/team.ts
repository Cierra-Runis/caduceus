import * as z from 'zod';

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
