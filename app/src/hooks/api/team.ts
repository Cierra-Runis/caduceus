'use client';

import { HTTPError } from 'ky';
import useSWR from 'swr';
import useSWRMutation from 'swr/mutation';

import {
  CreateTeamRequest,
  CreateTeamResponse,
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
  >('team', (key, { arg }) => api.post(key, { json: arg }).json());
};

export const useTeamProject = ({ id }: { id: string }) =>
  useSWR<TeamProjectResponse, HTTPError, string>('team/projects', async (key) =>
    TeamProjectResponseSchema.parse(
      await api
        .get(key, {
          searchParams: { id },
        })
        .json(),
    ),
  );
