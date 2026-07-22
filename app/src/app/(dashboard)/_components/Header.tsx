'use client';

import { CloudIcon } from 'lucide-react';
import { useTranslations } from 'next-intl';
import { useParams } from 'next/navigation';

import { Badge } from '@/components/ui/badge';
import { useUserMe } from '@/hooks/api/user/me';
import { useUserTeams } from '@/hooks/api/user/team';

// Route-aware workspace label in the middle of the navbar: the team's name
// on a team page, the user's own name everywhere else, falling back to the
// app name while either is still loading. `id` is `/dashboard/team/[id]`'s
// param — the only dynamic segment under this layout — so its presence is
// what marks a team page.
export function Header() {
  const t = useTranslations('Layout');
  const { id } = useParams<{ id?: string }>();
  const { data: user } = useUserMe();
  const { data: teams } = useUserTeams();

  const team = id ? teams?.payload.find((team) => team.id === id) : undefined;

  return (
    <Badge variant='ghost'>
      <CloudIcon className='w-[1.25em]' data-icon='inline-start' />
      {team?.name ?? user?.payload.username ?? t('caduceus')}
    </Badge>
  );
}
