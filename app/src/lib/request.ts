import ky from 'ky';

import { env } from '@/lib/env';

export const api = ky.extend({
  credentials: 'include',
  hooks: {
    beforeError: [
      async (error) => {
        const { response } = error;
        try {
          const res = await response.json<{
            message: string;
          }>();
          error.message = res.message;
          return error;
        } catch {
          return error;
        }
      },
    ],
  },
  prefixUrl: env.NEXT_PUBLIC_API_URL,
});
