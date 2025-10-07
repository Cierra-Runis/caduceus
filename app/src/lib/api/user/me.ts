import useSWR from 'swr';
import z from 'zod';

import { api } from '@/lib/request';
import { User } from '@/lib/types/user';

export type RouteUserMe = z.infer<typeof RouteUserMe>;
export const RouteUserMe = z.object({
  message: z.string(),
  payload: User,
});

export const useUserMe = () =>
  useSWR('user/me', async () =>
    RouteUserMe.parse(await api.get('user/me').json()),
  );
