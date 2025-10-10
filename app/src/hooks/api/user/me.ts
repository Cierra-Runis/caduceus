'use client';

import useSWR from 'swr';

import { RouteUserMe, RouteUserMeSchema } from '@/lib/api/user/me';
import { api } from '@/lib/request';

export const useUserMe = () =>
  useSWR<RouteUserMe>('user/me', async () =>
    RouteUserMeSchema.parse(await api.get('user/me').json()),
  );
