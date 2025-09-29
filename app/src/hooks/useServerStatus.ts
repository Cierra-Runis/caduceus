import { BadgeProps } from '@heroui/badge';
import { useMemo } from 'react';
import useSWR from 'swr';

import { api } from '@/lib/request';
import { ApiResponse, ErrorResponse } from '@/lib/response';

export interface ServerStatus {
  status: 'healthy';
  timestamp: string;
}

type HealthResponse = ApiResponse<ServerStatus>;

export function useServerStatus() {
  const { data, error, isLoading } = useSWR<
    HealthResponse,
    ErrorResponse,
    string
  >('health', (key) => api.get(key).json());

  const color = useMemo<BadgeProps['color']>(() => {
    if (isLoading) return 'default';
    if (error) return 'danger';
    if (data?.payload?.status === 'healthy') return 'success';
    return 'warning';
  }, [error, data, isLoading]);

  return { color };
}
