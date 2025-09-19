import z from 'zod';

import { ApiResponse } from '../response';
import { AuthPayload } from './register';

export const LoginSchema = z.object({
  password: z.string('Password is required').nonempty('Password is required'),
  username: z.string('Username is required').nonempty('Username is required'),
});

export type LoginRequest = z.infer<typeof LoginSchema>;
export type LoginResponse = ApiResponse<AuthPayload>;
