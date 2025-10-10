import * as z from 'zod';

export type User = z.infer<typeof UserSchema>;
export const UserSchema = z.object({
  avatar_uri: z.string().nullable(),
  created_at: z.string().transform((str) => new Date(str)),
  id: z.string(),
  nickname: z.string(),
  updated_at: z.string().transform((str) => new Date(str)),
  username: z.string(),
});
