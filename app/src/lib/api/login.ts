import z from 'zod';

import { ApiResponse } from '../response';
import { AuthPayload } from './register';

export const LoginSchema = z.object({
  password: z.string().nonempty('Password is required'),
  username: z.string().nonempty('Username is required'),
});

export type LoginRequest = z.infer<typeof LoginSchema>;
export type LoginResponse = ApiResponse<AuthPayload>;
