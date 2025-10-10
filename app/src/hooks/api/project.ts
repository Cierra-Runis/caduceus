'use client';

import useSWRMutation from 'swr/mutation';

import { CreateProjectRequest, CreateProjectResponse } from '@/lib/api/project';
import { api } from '@/lib/request';

export const useCreateProject = () => {
  return useSWRMutation<
    CreateProjectResponse,
    Error,
    string,
    CreateProjectRequest
  >('project', (key, { arg }) => api.post(key, { json: arg }).json());
};
