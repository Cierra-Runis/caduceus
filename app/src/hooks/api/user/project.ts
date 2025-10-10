import { HTTPError } from 'ky';
import useSWR from 'swr';

import {
  UserProjectResponse,
  UserProjectResponseSchema,
} from '@/lib/api/user/project';
import { api } from '@/lib/request';

export const useUserProject = () =>
  useSWR<UserProjectResponse, HTTPError, string>('user/projects', async (key) =>
    UserProjectResponseSchema.parse(await api.get(key).json()),
  );
