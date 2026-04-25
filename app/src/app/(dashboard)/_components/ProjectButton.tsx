'use client';

import { useTranslations } from 'next-intl';

import { Button } from '@/components/ui/button';
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from '@/components/ui/tooltip';

export function ProjectButton() {
  const t = useTranslations('Layout');
  return (
    <Tooltip>
      <TooltipTrigger>
        <Button>{t('project')}</Button>
      </TooltipTrigger>
      <TooltipContent align='start' side='bottom'>
        {
          //   <Listbox>
          //   <ListboxItem key='new-project'>{t('newProject')}</ListboxItem>
          //   <ListboxItem key='incoming-invites'>
          //     {t('incomingInvites')}
          //   </ListboxItem>
          // </Listbox>
        }
      </TooltipContent>
    </Tooltip>
  );
}
