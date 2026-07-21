'use client';

import useSWRMutation from 'swr/mutation';

import {
  CreateProjectRequest,
  CreateProjectResponse,
  CreateProjectResponseSchema,
  ProjectDetailResponse,
  ProjectDetailResponseSchema,
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

// On-demand fetch of a single project's detail (full file tree + entry). The
// dashboard's project list only carries `ProjectPayload` summaries, so
// anything needing file contents (e.g. compiling for download) triggers this
// manually rather than reading it off a cached SWR key.
export const useProjectDetail = () =>
  useSWRMutation<ProjectDetailResponse, Error, string, string>(
    'project',
    async (key, { arg: id }) =>
      ProjectDetailResponseSchema.parse(
        await api.get(`${key}/${id}`).json(),
      ),
  );
