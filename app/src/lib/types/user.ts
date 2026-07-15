import * as z from 'zod';

export type User = z.infer<typeof UserSchema>;
export const UserSchema = z.object({
  avatar_uri: z.string().trim().nullable(),
  created_at: z.string().trim().transform((str) => new Date(str)),
  id: z.string().trim(),
  nickname: z.string().trim(),
  updated_at: z.string().trim().transform((str) => new Date(str)),
  username: z.string().trim(),
});
