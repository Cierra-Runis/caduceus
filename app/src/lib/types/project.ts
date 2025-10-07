import z from 'zod';

export type Project = z.infer<typeof Project>;
export const Project = z.object({
  created_at: z.string().transform((str) => new Date(str)),
  creator_id: z.string(),
  id: z.string(),
  name: z.string(),
  owner_id: z.string(),
  owner_type: z.enum(['team', 'user']),
  updated_at: z.string().transform((str) => new Date(str)),
});
