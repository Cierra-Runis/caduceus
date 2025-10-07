import { HTTPError } from 'ky';
import useSWRMutation from 'swr/mutation';
import * as z from 'zod';

import { api } from '@/lib/request';
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

export const useCreateTeam = () => {
  return useSWRMutation<
    CreateTeamResponse,
    HTTPError,
    string,
    CreateTeamRequest
  >('team', (key, { arg }) => api.post(key, { json: arg }).json());
};
