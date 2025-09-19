import { addToast } from '@heroui/toast';
import { useRouter } from 'next/navigation';
import useSWRMutation from 'swr/mutation';

import { RegisterRequest, RegisterResponse } from '@/lib/api/register';
import { api } from '@/lib/request';
import { ErrorResponse } from '@/lib/response';

export const useRegister = () => {
  const router = useRouter();

  return useSWRMutation<
    RegisterResponse,
    ErrorResponse,
    string,
    RegisterRequest
  >('/api/register', (key, { arg }) => api.post(key, { json: arg }).json(), {
    onError: (error) => {
      addToast({
        color: 'danger',
        description: error.message,
        title: 'Register Failed',
      });
    },
    onSuccess: ({
      payload: {
        user: { username },
      },
    }) => {
      addToast({
        color: 'success',
        description: `Redirecting to login page...`,
        onClose: () => router.push('/login'), // FIXME: https://github.com/heroui-inc/heroui/issues/5609
        shouldShowTimeoutProgress: true,
        timeout: 3000,
        title: `Welcome, ${username}`,
      });
    },
  });
};
