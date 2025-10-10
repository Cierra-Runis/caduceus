'use client';

import { HTTPError } from 'ky';
import useSWRMutation from 'swr/mutation';

import { LoginRequest, LoginResponse } from '@/lib/api/login';
import { api } from '@/lib/request';

export const useLogin = () => {
  return useSWRMutation<LoginResponse, HTTPError, string, LoginRequest>(
    'login',
    (key, { arg }) => api.post(key, { json: arg }).json(),
  );
};
