import useSWR from 'swr';

import { api } from '@/lib/request';
import { ApiResponse, ErrorResponse } from '@/lib/response';

export type Team = {
  avatar_uri?: string;
  id: string;
  name: string;
};

type UserTeamsResponse = ApiResponse<Team[]>;

export function useUserTeams() {
  return useSWR<UserTeamsResponse, ErrorResponse, string>(
    '/api/user/teams',
    (key) => api.get(key).json(),
  );
}
