'use client';

import { useTranslations } from 'next-intl';

import { Button } from '@/components/ui/button';
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from '@/components/ui/tooltip';

export function HelpButton() {
  const t = useTranslations('Layout');
  return (
    <Tooltip>
      <TooltipTrigger>
        <Button>{t('help')}</Button>
      </TooltipTrigger>
      <TooltipContent align='start' side='bottom'>
        {/* <Listbox>
          <ListboxItem
            as={NextLink}
            href='https://typst.app/docs/tutorial'
            key='tutorial'
          >
            {t('tutorial')}
          </ListboxItem>
          <ListboxItem
            as={NextLink}
            href='https://typst.app/docs/reference/'
            key='reference'
          >
            {t('reference')}
          </ListboxItem>
          <ListboxItem key='feedback'>{t('feedback')}</ListboxItem>
          <ListboxItem as={NextLink} href='/contact' key='contact'>
            {t('contact')}
          </ListboxItem>
        </Listbox>   */}
      </TooltipContent>
    </Tooltip>
  );
}
