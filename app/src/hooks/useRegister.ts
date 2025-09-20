import useSWRMutation from 'swr/mutation';

import { RegisterRequest, RegisterResponse } from '@/lib/api/register';
import { api } from '@/lib/request';
import { ErrorResponse } from '@/lib/response';

export const useRegister = () => {
  return useSWRMutation<
    RegisterResponse,
    ErrorResponse,
    string,
    RegisterRequest
  >('/api/register', (key, { arg }) => api.post(key, { json: arg }).json());
};
