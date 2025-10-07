import z from 'zod';

export type User = z.infer<typeof User>;
export const User = z.object({
  avatar_uri: z.string().optional(),
  createAt: z.string().transform((str) => new Date(str)),
  id: z.string(),
  nickname: z.string(),
  updateAt: z.string().transform((str) => new Date(str)),
  username: z.string(),
});
