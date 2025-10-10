import * as z from 'zod';

import { UserSchema } from '@/lib/types/user';

export type RouteUserMe = z.infer<typeof RouteUserMeSchema>;
export const RouteUserMeSchema = z.object({
  message: z.string(),
  payload: UserSchema,
});
