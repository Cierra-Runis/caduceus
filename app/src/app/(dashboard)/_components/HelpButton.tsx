'use client';

import { Button } from '@heroui/button';
import { Listbox, ListboxItem } from '@heroui/listbox';
import { Tooltip } from '@heroui/tooltip';
import { useTranslations } from 'next-intl';
import NextLink from 'next/link';

export function HelpButton() {
  const t = useTranslations('Layout');
  return (
    <Tooltip
      content={
        <Listbox>
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
        </Listbox>
      }
      placement='bottom-start'
    >
      <Button>{t('help')}</Button>
    </Tooltip>
  );
}
