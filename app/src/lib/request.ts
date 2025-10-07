import ky from 'ky';

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
          return { ...error, message: res.message };
        } catch {
          return error;
        }
      },
    ],
  },
  prefixUrl: process.env.NEXT_PUBLIC_API_URL,
});
