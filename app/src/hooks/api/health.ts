'use client';

import useSWR from 'swr';

import { HealthResponseSchema } from '@/lib/api/health';
import { api } from '@/lib/request';

export const useRouteHealth = () =>
  useSWR('health', async () =>
    HealthResponseSchema.parse(await api.get('health').json()),
  );
