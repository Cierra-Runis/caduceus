import useSWR from 'swr';
import * as z from 'zod';

import { api } from '@/lib/request';

export type HealthResponse = z.infer<typeof HealthResponseSchema>;
export const HealthResponseSchema = z.object({
  message: z.string(),
  payload: z.object({
    status: z.enum(['healthy']),
  }),
});

export const useRouteHealth = () =>
  useSWR('health', async () =>
    HealthResponseSchema.parse(await api.get('health').json()),
  );
