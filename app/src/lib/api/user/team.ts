import useSWR from 'swr';
import z from 'zod';

import { api } from '@/lib/request';
import { Team } from '@/lib/types/team';

export type RouteUserTeams = z.infer<typeof RouteUserTeams>;
export const RouteUserTeams = z.object({
  message: z.string(),
  payload: z.array(Team),
});

export const useUserTeams = () =>
  useSWR('user/teams', async () =>
    RouteUserTeams.parse(await api.get('user/teams').json()),
  );
