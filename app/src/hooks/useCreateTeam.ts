import useSWRMutation from 'swr/mutation';

import { CreateTeamRequest, CreateTeamResponse } from '@/lib/api/team';
import { api } from '@/lib/request';
import { ErrorResponse } from '@/lib/response';

export const useCreateTeam = () => {
  return useSWRMutation<
    CreateTeamResponse,
    ErrorResponse,
    string,
    CreateTeamRequest
  >('team', (key, { arg }) => api.post(key, { json: arg }).json());
};
