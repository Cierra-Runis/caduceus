'use client';

import { CloudIcon } from 'lucide-react';
import { useTranslations } from 'next-intl';

import { Badge } from '@/components/ui/badge';
import { useUserMe } from '@/hooks/api/user/me';
import { useUserTeams } from '@/hooks/api/user/team';

// Workspace label in the middle of the navbar. Which variant renders for
// which route is decided by the `@header` parallel-route slot (see
// `../@header`), so the label itself never inspects the URL: team pages
// mount `TeamHeader` with a compiler-checked id from their route params,
// everything else renders `UserHeader`.

export function TeamHeader({ teamId }: { teamId: string }) {
  const t = useTranslations('Layout');
  const { data: teams } = useUserTeams();
  const team = teams?.payload.find((team) => team.id === teamId);
  return <HeaderBadge>{team?.name ?? t('caduceus')}</HeaderBadge>;
}

export function UserHeader() {
  const t = useTranslations('Layout');
  const { data: user } = useUserMe();
  return <HeaderBadge>{user?.payload.username ?? t('caduceus')}</HeaderBadge>;
}

function HeaderBadge({ children }: { children: React.ReactNode }) {
  return (
    <Badge variant='ghost'>
      <CloudIcon className='w-[1.25em]' data-icon='inline-start' />
      {children}
    </Badge>
  );
}
