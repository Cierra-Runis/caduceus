'use client';

import { Button } from '@heroui/button';
import { Listbox, ListboxItem } from '@heroui/listbox';
import { Tooltip } from '@heroui/tooltip';
import { useTranslations } from 'next-intl';

export function ProjectButton() {
  const t = useTranslations('Layout');
  return (
    <Tooltip
      content={
        <Listbox>
          <ListboxItem key='new-project'>{t('newProject')}</ListboxItem>
          <ListboxItem key='incoming-invites'>
            {t('incomingInvites')}
          </ListboxItem>
        </Listbox>
      }
      placement='bottom-start'
    >
      <Button>{t('project')}</Button>
    </Tooltip>
  );
}
