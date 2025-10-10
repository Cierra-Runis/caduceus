import * as z from 'zod';

export const TeamSchema = z.object({
  avatar_uri: z.string().nullable(),
  id: z.string(),
  name: z.string(),
});
