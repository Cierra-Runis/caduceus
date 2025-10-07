import useSWR from 'swr';
import * as z from 'zod';

import { api } from '@/lib/request';
import { UserSchema } from '@/lib/types/user';

export type RouteUserMe = z.infer<typeof RouteUserMeSchema>;
export const RouteUserMeSchema = z.object({
  message: z.string(),
  payload: UserSchema,
});

export const useUserMe = () =>
  useSWR('user/me', async () =>
    RouteUserMeSchema.parse(await api.get('user/me').json()),
  );
