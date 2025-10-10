'use client';

import { Chip } from '@heroui/chip';
import { IconCloud } from '@tabler/icons-react';
import { useTranslations } from 'next-intl';

import { useUserMe } from '@/hooks/api/user/me';

// TODO: Dynamic header info
export function Header() {
  const { data } = useUserMe();
  const t = useTranslations('Layout');
  return (
    <Chip startContent={<IconCloud className='w-[1.25em]' />} variant='light'>
      {data?.payload?.username ?? t('caduceus')}
    </Chip>
  );
}
