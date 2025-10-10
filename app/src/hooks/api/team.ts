'use client';

import { HTTPError } from 'ky';
import useSWRMutation from 'swr/mutation';

import { CreateTeamRequest, CreateTeamResponse } from '@/lib/api/team';
import { api } from '@/lib/request';

export const useCreateTeam = () => {
  return useSWRMutation<
    CreateTeamResponse,
    HTTPError,
    string,
    CreateTeamRequest
  >('team', (key, { arg }) => api.post(key, { json: arg }).json());
};
