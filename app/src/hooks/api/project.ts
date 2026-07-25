'use client';

import useSWRMutation from 'swr/mutation';

import {
  CreateProjectRequest,
  CreateProjectResponse,
  CreateProjectResponseSchema,
  DuplicateProjectResponse,
  DuplicateProjectResponseSchema,
  ProjectDetailResponse,
  ProjectDetailResponseSchema,
  UpdateProjectRequest,
  UpdateProjectResponse,
  UpdateProjectResponseSchema,
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

export const useProjectDetail = () =>
  useSWRMutation<ProjectDetailResponse, Error, string, string>(
    'project',
    async (key, { arg: id }) =>
      ProjectDetailResponseSchema.parse(
        await api.get(`${key}/${id}`).json(),
      ),
  );

export const useDuplicateProject = (id: string) =>
  useSWRMutation<DuplicateProjectResponse, Error, string>(
    `project/${id}/duplicate`,
    async (key) =>
      DuplicateProjectResponseSchema.parse(await api.post(key).json()),
  );

export const useUpdateProject = (id: string) =>
  useSWRMutation<UpdateProjectResponse, Error, string, UpdateProjectRequest>(
    `project/${id}`,
    async (key, { arg }) =>
      UpdateProjectResponseSchema.parse(
        await api.put(key, { json: arg }).json(),
      ),
  );
