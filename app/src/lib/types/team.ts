import * as z from 'zod';

export const TeamSchema = z.object({
  avatar_uri: z.string().trim().nullable(),
  id: z.string().trim(),
  name: z.string().trim(),
});
