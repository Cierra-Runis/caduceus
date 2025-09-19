import z from 'zod';

import { AuthPayload } from '../models/user';
import { ApiResponse } from '../response';

export const RegisterRequest = z
  .object({
    confirmPassword: z
      .string('Confirm Password is required')
      .nonempty('Confirm Password is required'),
    nickname: z.string().optional(),
    password: z
      .string('Password is required')
      .regex(
        /^(?=.{15,}$)|(?=.*[a-z])(?=.*[A-Z])(?=.*\d).{8,}$/,
        'Password should be at least 15 characters OR at least 8 characters including a number and a lowercase letter.',
      ),
    username: z
      .string('Username is required')
      .regex(
        /^(?!-)[a-zA-Z0-9-]{1,39}(?<!-)$/,
        'Username may only contain alphanumeric characters or single hyphens, and cannot begin or end with a hyphen.',
      ),
  })
  .refine((data) => data.password === data.confirmPassword, {
    error: 'Passwords do not match',
    path: ['confirmPassword'],
  });

export type RegisterRequest = z.infer<typeof RegisterRequest>;
export type RegisterResponse = ApiResponse<AuthPayload>;
