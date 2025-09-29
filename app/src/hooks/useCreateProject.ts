import useSWRMutation from 'swr/mutation';

import { CreateProjectRequest, CreateProjectResponse } from '@/lib/api/project';
import { api } from '@/lib/request';
import { ErrorResponse } from '@/lib/response';

export const useCreateProject = () => {
  return useSWRMutation<
    CreateProjectResponse,
    ErrorResponse,
    string,
    CreateProjectRequest
  >('project', (key, { arg }) => api.post(key, { json: arg }).json());
};
