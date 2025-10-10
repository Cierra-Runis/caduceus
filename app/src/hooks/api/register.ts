'use client';

import { HTTPError } from 'ky';
import useSWRMutation from 'swr/mutation';

import {
  RegisterRequest,
  RegisterResponse,
  RegisterResponseSchema,
} from '@/lib/api/register';
import { api } from '@/lib/request';

export const useRegister = () =>
  useSWRMutation<RegisterResponse, HTTPError, string, RegisterRequest>(
    'register',
    async (key, { arg }) =>
      RegisterResponseSchema.parse(await api.post(key, { json: arg }).json()),
  );
