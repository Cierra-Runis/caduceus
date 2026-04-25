'use client';

import { useTranslations } from 'next-intl';

import { Button } from '@/components/ui/button';
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from '@/components/ui/tooltip';

export function CaduceusButton() {
  const t = useTranslations('Layout');
  return (
    <Tooltip>
      <TooltipTrigger>
        <Button className='font-bold'>{t('caduceus')}</Button>
      </TooltipTrigger>
      <TooltipContent align='start' side='bottom'>
        {/* <Listbox>
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
        </Listbox> */}
      </TooltipContent>
    </Tooltip>
  );
}
