import useSWR from 'swr';

import { ApiResponse } from '@/lib/response';

const fetcher = (url: string) => fetch(url).then((r) => r.json());

export type Team = {
  avatar_uri?: string;
  id: string;
  name: string;
};

export function useUserTeams() {
  const { data, error, isLoading } = useSWR<ApiResponse<Team[]>>(
    '/api/user/teams',
    fetcher,
  );

  return {
    isError: error,
    isLoading,
    teams: data,
  };
}
