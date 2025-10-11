'use client';

import { HTTPError } from 'ky';
import useSWRMutation from 'swr/mutation';

import {
  LoginRequest,
  LoginResponse,
  LoginResponseSchema,
} from '@/lib/api/login';
import { api } from '@/lib/request';

export const useLogin = () =>
  useSWRMutation<LoginResponse, HTTPError, string, LoginRequest>(
    'login',
    async (key, { arg }) =>
      LoginResponseSchema.parse(await api.post(key, { json: arg }).json()),
  );
