'use client';

import useSWR from 'swr';

import { RouteUserTeamsSchema } from '@/lib/api/user/team';
import { api } from '@/lib/request';

export const useUserTeams = () =>
  useSWR('user/teams', async () =>
    RouteUserTeamsSchema.parse(await api.get('user/teams').json()),
  );
