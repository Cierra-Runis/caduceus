import * as z from 'zod';

import { ApiResponse } from '../response';

export const CreateTeamRequest = z.object({
  name: z.string('Team name is required').nonempty('Team name is required'),
});

export type CreateTeamRequest = z.infer<typeof CreateTeamRequest>;
export type CreateTeamResponse = ApiResponse<TeamPayload>;

export interface TeamPayload {
  avatar_uri?: string;
  id: string;
  name: string;
}
