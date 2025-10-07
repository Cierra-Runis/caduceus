import useSWR from 'swr';
import * as z from 'zod';

import { api } from '@/lib/request';
import { TeamSchema } from '@/lib/types/team';

export type RouteUserTeams = z.infer<typeof RouteUserTeamsSchema>;
export const RouteUserTeamsSchema = z.object({
  message: z.string(),
  payload: z.array(TeamSchema),
});

export const useUserTeams = () =>
  useSWR('user/teams', async () =>
    RouteUserTeamsSchema.parse(await api.get('user/teams').json()),
  );
