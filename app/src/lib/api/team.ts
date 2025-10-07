import { HTTPError } from 'ky';
import useSWRMutation from 'swr/mutation';
import * as z from 'zod';

import { api } from '@/lib/request';
import { Team } from '@/lib/types/team';

export const CreateTeamRequest = z.object({
  name: z.string('Team name is required').nonempty('Team name is required'),
});
export type CreateTeamRequest = z.infer<typeof CreateTeamRequest>;

export const CreateTeamResponse = z.object({
  message: z.string(),
  payload: Team,
});
export type CreateTeamResponse = z.infer<typeof CreateTeamResponse>;

export const useCreateTeam = () => {
  return useSWRMutation<
    CreateTeamResponse,
    HTTPError,
    string,
    CreateTeamRequest
  >('team', (key, { arg }) => api.post(key, { json: arg }).json());
};
