'use client';

import useSWRMutation from 'swr/mutation';

import {
  CreateProjectRequest,
  CreateProjectResponse,
  CreateProjectResponseSchema,
  DuplicateProjectResponse,
  DuplicateProjectResponseSchema,
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

export const useDuplicateProject = (id: string) =>
  useSWRMutation<DuplicateProjectResponse, Error, string>(
    `project/${id}/duplicate`,
    async (key) =>
      DuplicateProjectResponseSchema.parse(await api.post(key).json()),
  );
