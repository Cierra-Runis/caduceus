import z from 'zod/mini';

import { ApiResponse } from '../response';

export const RegisterRequest = z
  .object({
    confirmPassword: z.string('Confirm Password is required'),
    nickname: z.optional(z.string()),
    password: z
      .string('Password is required')
      .check(
        z.regex(
          /^(?=.{15,}$)|(?=.*[a-z])(?=.*[A-Z])(?=.*\d).{8,}$/,
          'Password should be at least 15 characters OR at least 8 characters including a number and a lowercase letter.',
        ),
      ),
    username: z
      .string('Username is required')
      .check(
        z.regex(
          /^(?!-)[a-zA-Z0-9-]{1,39}(?<!-)$/,
          'Username may only contain alphanumeric characters or single hyphens, and cannot begin or end with a hyphen.',
        ),
      ),
  })
  .check(
    z.refine((data) => data.password === data.confirmPassword, {
      error: 'Passwords do not match',
      path: ['confirmPassword'],
    }),
  );
export interface AuthPayload {
  token: string;
  user: UserPayload;
}

export type RegisterRequest = z.infer<typeof RegisterRequest>;

export type RegisterResponse = ApiResponse<AuthPayload>;

export interface UserPayload {
  createAt: Date;
  id: string;
  nickname: string;
  updateAt: Date;
  username: string;
}
