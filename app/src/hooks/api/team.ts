'use client';

import { HTTPError } from 'ky';
import useSWR from 'swr';
import useSWRMutation from 'swr/mutation';

import {
  CreateTeamRequest,
  CreateTeamResponse,
  CreateTeamResponseSchema,
  TeamProjectResponse,
  TeamProjectResponseSchema,
} from '@/lib/api/team';
import { api } from '@/lib/request';

export const useCreateTeam = () => {
  return useSWRMutation<
    CreateTeamResponse,
    HTTPError,
    string,
    CreateTeamRequest
  >('team', async (key, { arg }) =>
    CreateTeamResponseSchema.parse(await api.post(key, { json: arg }).json()),
  );
};

// SWR key for a team's project list. The team id is part of the key so each
// team gets its own cache entry, and the key doubles as the request path.
// Mutating call sites must build the key through this helper too.
export const teamProjectKey = (id: string) => `team/projects?id=${id}`;

export const useTeamProject = ({ id }: { id: string }) =>
  useSWR<TeamProjectResponse, HTTPError, string>(
    teamProjectKey(id),
    async (key) => TeamProjectResponseSchema.parse(await api.get(key).json()),
  );
