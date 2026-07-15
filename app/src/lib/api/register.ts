import { useTranslations } from 'next-intl';
import * as z from 'zod';

import { UserSchema } from '../types/user';

export const useRegisterRequestSchema = () => {
  const t = useTranslations('Register.validation');
  return z
    .object({
      confirmPassword: z
        .string(t('confirmPassword')).trim()
        .nonempty(t('confirmPassword')),
      nickname: z.string().trim().optional(),
      password: z
        .string(t('password')).trim()
        .regex(
          /^(?=.{15,}$)|(?=.*[a-z])(?=.*[A-Z])(?=.*\d).{8,}$/,
          t('password'),
        ),
      username: z
        .string(t('username')).trim()
        .regex(/^(?!-)[a-zA-Z0-9-]{1,39}(?<!-)$/, t('username')),
    })
    .refine((data) => data.password === data.confirmPassword, {
      error: t('confirmPassword'),
      path: ['confirmPassword'],
    });
};

export type RegisterRequest = z.infer<
  ReturnType<typeof useRegisterRequestSchema>
>;
export type RegisterResponse = z.infer<typeof RegisterResponseSchema>;
export const RegisterResponseSchema = z.object({
  message: z.string().trim(),
  payload: z.object({
    token: z.string().trim(),
    user: UserSchema,
  }),
});
