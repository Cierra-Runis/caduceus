'use client';

import useSWRMutation from 'swr/mutation';

import {
  CreateProjectRequest,
  CreateProjectResponse,
  CreateProjectResponseSchema,
} from '@/lib/api/project';
import { api } from '@/lib/request';

export const useCreateProject = () =>
  useSWRMutation<CreateProjectResponse, Error, string, CreateProjectRequest>(
    'project',
    async (key, { arg }) =>
      CreateProjectResponseSchema.parse(
        await api.post(key, { json: arg }).json(),
      ),
  );
