import * as z from 'zod';

import { ProjectSchema } from '@/lib/types/project';

export type UserProjectResponse = z.infer<typeof UserProjectResponseSchema>;
export const UserProjectResponseSchema = z.object({
  payload: z.array(ProjectSchema),
});
