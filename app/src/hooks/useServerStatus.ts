import { BadgeProps } from '@heroui/badge';
import { useMemo } from 'react';

import { useRouteHealth } from '@/lib/api/health';

export function useServerStatus() {
  const { data, error, isLoading } = useRouteHealth();

  const color = useMemo<BadgeProps['color']>(() => {
    if (isLoading) return 'default';
    if (error) return 'danger';
    if (data?.payload?.status === 'healthy') return 'success';
    return 'warning';
  }, [error, data, isLoading]);

  return { color };
}
