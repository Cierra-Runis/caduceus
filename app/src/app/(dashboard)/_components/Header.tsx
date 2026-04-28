'use client';

import { CloudIcon } from 'lucide-react';
import { useTranslations } from 'next-intl';

import { Badge } from '@/components/ui/badge';
import { useUserMe } from '@/hooks/api/user/me';

// TODO: Dynamic header info
export function Header() {
  const { data } = useUserMe();
  const t = useTranslations('Layout');
  return (
    <Badge variant='ghost'>
      <CloudIcon className='w-[1.25em]' data-icon='inline-start' />
      {data?.payload?.username ?? t('caduceus')}
    </Badge>
  );
}
