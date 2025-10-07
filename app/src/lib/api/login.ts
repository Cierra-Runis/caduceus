import { HTTPError } from 'ky';
import useSWRMutation from 'swr/mutation';
import * as z from 'zod';

import { api } from '../request';
import { UserSchema } from '../types/user';

export type LoginRequest = z.infer<typeof LoginSchema>;
export const LoginSchema = z.object({
  password: z.string('Password is required').nonempty('Password is required'),
  username: z.string('Username is required').nonempty('Username is required'),
});

export type LoginResponse = z.infer<typeof LoginResponseSchema>;
export const LoginResponseSchema = z.object({
  message: z.string(),
  payload: z.object({
    token: z.string(),
    user: UserSchema,
  }),
});

export const useLogin = () => {
  return useSWRMutation<LoginResponse, HTTPError, string, LoginRequest>(
    'login',
    (key, { arg }) => api.post(key, { json: arg }).json(),
  );
};
