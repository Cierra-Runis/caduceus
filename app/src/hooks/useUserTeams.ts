import useSWR from 'swr';

import { TeamPayload } from '@/lib/api/team';
import { api } from '@/lib/request';
import { ApiResponse, ErrorResponse } from '@/lib/response';

type UserTeamsResponse = ApiResponse<TeamPayload[]>;

export function useUserTeams() {
  return useSWR<UserTeamsResponse, ErrorResponse, string>(
    '/api/user/teams',
    (key) => api.get(key).json(),
  );
}
