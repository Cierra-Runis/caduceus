import { BadgeProps } from '@heroui/badge';
import { useMemo } from 'react';
import useSWR from 'swr';

export type ServerStatus = {
  message: string;
  payload: {
    status: 'ok';
    timestamp: string;
  };
};

const fetcher = (url: string) => fetch(url).then((r) => r.json());

export function useServerStatus() {
  const { error, isLoading, data } = useSWR<ServerStatus>(
    '/api/health',
    fetcher
  );

  const color = useMemo<BadgeProps['color']>(() => {
    if (isLoading) return 'default';
    if (error) return 'danger';
    if (data?.payload?.status === 'ok') return 'success';
    return 'warning';
  }, [error, data, isLoading]);

  return { color };
}
