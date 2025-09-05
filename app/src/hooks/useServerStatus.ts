import { BadgeProps } from '@heroui/badge';
import { useMemo } from 'react';
import useSWR from 'swr';

import { ApiResponse } from '@/lib/response';

export type ServerStatus = {
  status: 'healthy';
  timestamp: string;
};

const fetcher = (url: string) => fetch(url).then((r) => r.json());

export function useServerStatus() {
  const { data, error, isLoading } = useSWR<ApiResponse<ServerStatus>>(
    '/api/health',
    fetcher,
  );

  const color = useMemo<BadgeProps['color']>(() => {
    if (isLoading) return 'default';
    if (error) return 'danger';
    if (data?.payload?.status === 'healthy') return 'success';
    return 'warning';
  }, [error, data, isLoading]);

  return { color };
}
