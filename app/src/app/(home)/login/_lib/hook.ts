import { addToast } from '@heroui/toast';
import { useRouter } from 'next/navigation';
import useSWRMutation from 'swr/mutation';

import { LoginRequest, LoginResponse } from '@/lib/api/login';
import { api } from '@/lib/request';
import { ErrorResponse } from '@/lib/response';

export const useLogin = () => {
  const router = useRouter();

  return useSWRMutation<LoginResponse, ErrorResponse, string, LoginRequest>(
    '/api/login',
    (key, { arg }) => api.post(key, { json: arg }).json(),
    {
      onError: (error) => {
        addToast({
          color: 'danger',
          description: error.message,
          title: 'Login Failed',
        });
      },
      onSuccess: ({
        payload: {
          user: { username },
        },
      }) => {
        addToast({
          color: 'success',
          description: 'Redirecting to homepage...',
          onClose: () => router.push('/'), // FIXME: https://github.com/heroui-inc/heroui/issues/5609
          shouldShowTimeoutProgress: true,
          timeout: 3000,
          title: `Welcome, ${username}`,
        });
      },
    },
  );
};
