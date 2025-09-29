import useSWR from 'swr';

import { UserPayload } from '@/lib/api/register';
import { api } from '@/lib/request';
import { ApiResponse, ErrorResponse } from '@/lib/response';

type UserMeResponse = ApiResponse<UserPayload>;

export function useUserMe() {
  return useSWR<UserMeResponse, ErrorResponse, string>('user/me', (key) =>
    api.get(key).json(),
  );
}
