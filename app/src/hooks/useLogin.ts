import useSWRMutation from 'swr/mutation';

import { LoginRequest, LoginResponse } from '@/lib/api/login';
import { api } from '@/lib/request';
import { ErrorResponse } from '@/lib/response';

export const useLogin = () => {
  return useSWRMutation<LoginResponse, ErrorResponse, string, LoginRequest>(
    '/api/login',
    (key, { arg }) => api.post(key, { json: arg }).json(),
  );
};
