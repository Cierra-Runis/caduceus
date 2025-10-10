'use client';

import { Button } from '@heroui/button';
import { Listbox, ListboxItem } from '@heroui/listbox';
import { Tooltip } from '@heroui/tooltip';
import { useTranslations } from 'next-intl';
import NextLink from 'next/link';

import { logout } from '@/actions/auth';

export function CaduceusButton() {
  const t = useTranslations('Layout');
  return (
    <Tooltip
      content={
        <Listbox>
          <ListboxItem as={NextLink} href='/about' key='about'>
            {t('about')}
          </ListboxItem>
          <ListboxItem as={NextLink} href='/dashboard/settings' key='settings'>
            {t('accountSettings')}
          </ListboxItem>
          <ListboxItem key='logout' onPress={logout}>
            {t('logout')}
          </ListboxItem>
          <ListboxItem as={NextLink} href='/home' key='home'>
            {t('goToLanding')}
          </ListboxItem>
        </Listbox>
      }
      placement='bottom-start'
    >
      <Button className='font-bold'>{t('caduceus')}</Button>
    </Tooltip>
  );
}
