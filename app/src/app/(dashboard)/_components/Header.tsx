'use client';

import { CloudIcon } from 'lucide-react';
import { useTranslations } from 'next-intl';
import { usePathname } from 'next/navigation';

import { Badge } from '@/components/ui/badge';
import { useUserMe } from '@/hooks/api/user/me';
import { useUserTeams } from '@/hooks/api/user/team';

// Route-aware workspace label in the middle of the navbar: the team's name
// on a team page, the user's own name everywhere else, falling back to the
// app name while either is still loading.
export function Header() {
  const t = useTranslations('Layout');
  const pathname = usePathname();
  const { data: user } = useUserMe();
  const { data: teams } = useUserTeams();

  const teamId = /^\/dashboard\/team\/([^/]+)/.exec(pathname)?.[1];
  const team = teamId
    ? teams?.payload.find((team) => team.id === teamId)
    : undefined;

  return (
    <Badge variant='ghost'>
      <CloudIcon className='w-[1.25em]' data-icon='inline-start' />
      {team?.name ?? user?.payload.username ?? t('caduceus')}
    </Badge>
  );
}
