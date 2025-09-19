import z from 'zod/mini';

import { ApiResponse } from '../response';
import { AuthPayload } from './register';

export const LoginSchema = z.object({
  password: z
    .string('Password is required')
    .check(z.minLength(1, 'Password is required')),
  username: z
    .string('Username is required')
    .check(z.minLength(1, 'Username is required')),
});

export type LoginRequest = z.infer<typeof LoginSchema>;
export type LoginResponse = ApiResponse<AuthPayload>;
