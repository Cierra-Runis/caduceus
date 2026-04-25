'use client';

import { CheckIcon, XIcon } from 'lucide-react';
import { useTranslations } from 'next-intl';
import { match } from 'ts-pattern';

import { ServerStatus, useServerStatus } from '@/hooks/useServerStatus';

import { Badge } from '../ui/badge';
import { Spinner } from '../ui/spinner';

export function ServerBadge() {
  const t = useTranslations();
  const { status } = useServerStatus();

  return match(status)
    .with(ServerStatus.Loading, () => (
      <Badge variant='ghost'>
        <Spinner data-icon='inline-start' />
        {t('ServiceStatus.loadingStatus')}
      </Badge>
    ))
    .with(ServerStatus.Healthy, () => (
      <Badge
        className={`
          text-green-700
          dark:text-green-300
        `}
        variant='ghost'
      >
        <CheckIcon data-icon='inline-start' />
        {t('ServiceStatus.allServicesAvailable')}
      </Badge>
    ))
    .with(ServerStatus.Unhealthy, () => (
      <Badge
        className={`
          text-yellow-700
          dark:text-yellow-300
        `}
        variant='ghost'
      >
        <XIcon data-icon='inline-start' />
        {t('ServiceStatus.someServicesUnavailable')}
      </Badge>
    ))
    .with(ServerStatus.Unavailable, () => (
      <Badge
        className={`
          text-red-700
          dark:text-red-300
        `}
        variant='ghost'
      >
        <XIcon data-icon='inline-start' />
        {t('ServiceStatus.noServiceAvailable')}
      </Badge>
    ))
    .exhaustive();
}
