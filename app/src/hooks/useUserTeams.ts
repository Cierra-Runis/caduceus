import useSWR from 'swr';

const fetcher = (url: string) => fetch(url).then((r) => r.json());

export type Team = {
  id: string;
  name: string;
};

export function useUserTeams() {
  const { data, error, isLoading } = useSWR<Team[]>('/api/user/teams', fetcher);

  return {
    isError: error,
    isLoading,
    teams: data,
  };
}
