import useSWR from 'swr';
import z from 'zod';

import { api } from '@/lib/request';

export type HealthResponse = z.infer<typeof HealthResponse>;
export const HealthResponse = z.object({
  message: z.string(),
  payload: z.object({
    status: z.enum(['healthy']),
  }),
});

export const useRouteHealth = () =>
  useSWR('health', async () =>
    HealthResponse.parse(await api.get('health').json()),
  );
