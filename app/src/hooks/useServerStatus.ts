import { BadgeProps } from '@heroui/badge';
import { useMemo } from 'react';
import useSWR from 'swr';

export type ServerStatus = {
  status: 'healthy';
  timestamp: string;
};

const fetcher = (url: string) => fetch(url).then((r) => r.json());

export function useServerStatus() {
  const { data, error, isLoading } = useSWR<ServerStatus>(
    '/api/health',
    fetcher,
  );

  const color = useMemo<BadgeProps['color']>(() => {
    if (isLoading) return 'default';
    if (error) return 'danger';
    if (data?.status === 'healthy') return 'success';
    return 'warning';
  }, [error, data, isLoading]);

  return { color };
}
