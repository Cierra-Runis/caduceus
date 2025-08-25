import { BadgeProps } from '@heroui/badge';
import { useMemo } from 'react';
import useSWR from 'swr';

export type ServerStatus = {
  message: string;
  data: {
    status: 'ok';
    timestamp: string;
  };
};

const fetcher = (url: string) => fetch(url).then((r) => r.json());

export function useServerStatus() {
  const { error, isLoading, data } = useSWR<ServerStatus>(
    '/api/health',
    fetcher,
    {
      refreshInterval: 100,
    }
  );

  const color = useMemo<BadgeProps['color']>(() => {
    if (isLoading) return 'default';
    if (error) return 'danger';
    if (data?.data?.status === 'ok') return 'success';
    return 'warning';
  }, [error, data, isLoading]);

  return { color };
}
