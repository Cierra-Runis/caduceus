import ky from 'ky';

import { ErrorResponse } from './response';

export const api = ky.extend({
  hooks: {
    beforeError: [
      async (error) => {
        console.log('API Error:', error);
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
});
