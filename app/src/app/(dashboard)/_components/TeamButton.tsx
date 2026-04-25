'use client';

import { useTranslations } from 'next-intl';

import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from '@/components/ui/tooltip';

export function TeamButton() {
  const t = useTranslations('Layout');
  return (
    <Tooltip>
      <TooltipTrigger>{t('team')}</TooltipTrigger>
      <TooltipContent align='start' side='bottom'>
        {/* <Listbox>
          <ListboxItem key='new-team'>{t('newTeam')}</ListboxItem>
          <ListboxItem key='manage-teams'>{t('manageTeams')}</ListboxItem>
          <ListboxItem key='incoming-invites'>
            {t('incomingInvites')}
          </ListboxItem>
        </Listbox> */}
      </TooltipContent>
    </Tooltip>
  );
}
