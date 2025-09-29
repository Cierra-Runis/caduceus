import ky from 'ky';

import { ErrorResponse } from './response';

export const api = ky.extend({
  credentials: 'include',
  hooks: {
    beforeError: [
      async (error) => {
        const { response } = error;
        try {
          const res = await response.json<ErrorResponse>();
          return { ...error, message: res.message };
        } catch {
          return error;
        }
      },
    ],
  },
  prefixUrl: process.env.NEXT_PUBLIC_API_URL,
});
