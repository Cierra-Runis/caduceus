import z from 'zod';

import { TeamPayload } from '../models/team';
import { ApiResponse } from '../response';

export const CreateTeamRequest = z.object({
  name: z.string('Team name is required').nonempty('Team name is required'),
});

export type CreateTeamRequest = z.infer<typeof CreateTeamRequest>;
export type CreateTeamResponse = ApiResponse<TeamPayload>;
