import * as z from 'zod';

import { TeamSchema } from '@/lib/types/team';

export type RouteUserTeams = z.infer<typeof RouteUserTeamsSchema>;
export const RouteUserTeamsSchema = z.object({
  message: z.string().trim(),
  payload: z.array(TeamSchema),
});
