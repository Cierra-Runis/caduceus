'use client';

import { HTTPError } from 'ky';
import useSWR from 'swr';

import { HealthResponse, HealthResponseSchema } from '@/lib/api/health';
import { api } from '@/lib/request';

export const useRouteHealth = () =>
  useSWR<HealthResponse, HTTPError, string>('health', async () =>
    HealthResponseSchema.parse(await api.get('health').json()),
  );
