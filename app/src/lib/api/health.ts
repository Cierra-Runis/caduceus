import * as z from 'zod';

export type HealthResponse = z.infer<typeof HealthResponseSchema>;
export const HealthResponseSchema = z.object({
  message: z.string(),
  payload: z.object({
    status: z.enum(['healthy']),
  }),
});
