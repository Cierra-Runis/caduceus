import { useMemo } from 'react';

import { useRouteHealth } from './api/health';

export enum ServerStatus {
  Healthy = 'healthy',
  Loading = 'loading',
  Unavailable = 'unavailable',
  Unhealthy = 'unhealthy',
}

export function useServerStatus() {
  const { data, error, isLoading } = useRouteHealth();

  const status = useMemo(() => {
    if (isLoading) return ServerStatus.Loading;
    if (error) return ServerStatus.Unavailable;
    if (data?.payload.status === 'healthy') return ServerStatus.Healthy;
    return ServerStatus.Unhealthy;
  }, [error, data, isLoading]);

  return { status };
}
