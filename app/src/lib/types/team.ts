import z from 'zod';

export const Team = z.object({
  avatar_uri: z.string().optional(),
  id: z.string(),
  name: z.string(),
});
