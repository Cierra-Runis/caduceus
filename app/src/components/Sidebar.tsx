'use client';

import { Avatar } from '@heroui/avatar';
import { Button } from '@heroui/button';
import { Tooltip } from '@heroui/tooltip';
import { IconLogout, IconSettings } from '@tabler/icons-react';
import NextLink from 'next/link';
import { useParams, usePathname } from 'next/navigation';

import { logout } from '@/actions/auth';
import { useUserTeams } from '@/hooks/useUserTeams';

import { CreateTeamButton } from './buttons/CreateTeamButton';

export function Sidebar() {
  const { team } = useParams();
  const pathname = usePathname();
  const { teams } = useUserTeams();

  const isInSettings =
    pathname.endsWith('/settings') || pathname.endsWith('/manage');

  return (
    <aside className='relative flex flex-col items-center justify-between pb-4 transition-all'>
      <div className='flex flex-col items-center'>
        <Button
          as={NextLink}
          className='h-16 w-16'
          href='/dashboard'
          isIconOnly
          radius='none'
          variant={!team ? 'solid' : 'light'}
        >
          <Avatar src='https://i.pravatar.cc?img=1' />
        </Button>

        {teams?.map((t) => (
          <Button
            as={NextLink}
            className='h-16 w-16'
            href={`/dashboard/team/${t.id}`}
            isIconOnly
            key={t.id}
            radius='none'
            variant={team === t.id ? 'solid' : 'light'}
          >
            <Tooltip content={t.name} placement='right'>
              <Avatar radius='sm' src={t.avatar_uri || '/icon.svg'} />
            </Tooltip>
          </Button>
        ))}

        <CreateTeamButton className='h-16 w-16' radius='none' variant='light' />
      </div>

      <div className='flex flex-col items-center space-y-2'>
        <Button
          as={NextLink}
          href={
            isInSettings
              ? team
                ? `/dashboard/team/${team}`
                : '/dashboard'
              : team
                ? `/dashboard/team/${team}/manage`
                : '/dashboard/settings'
          }
          isIconOnly
          startContent={<IconSettings />}
          variant={isInSettings ? 'solid' : 'light'}
        />
        <Button
          isIconOnly
          onPress={logout}
          startContent={<IconLogout />}
          variant='light'
        />
      </div>
    </aside>
  );
}
