import { useTranslations } from 'next-intl';
import * as z from 'zod';

import { UserSchema } from '../types/user';

export const useLoginRequestSchema = () => {
  const t = useTranslations('Login.validation');
  return z.object({
    password: z.string(t('password')).nonempty(t('password')),
    username: z.string(t('username')).nonempty(t('username')),
  });
};

export type LoginRequest = z.infer<ReturnType<typeof useLoginRequestSchema>>;
export type LoginResponse = z.infer<typeof LoginResponseSchema>;
export const LoginResponseSchema = z.object({
  message: z.string(),
  payload: z.object({
    token: z.string(),
    user: UserSchema,
  }),
});
