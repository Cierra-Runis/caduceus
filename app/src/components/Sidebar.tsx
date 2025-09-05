'use client';

import { Avatar } from '@heroui/avatar';
import { Button } from '@heroui/button';
import { ScrollShadow } from '@heroui/scroll-shadow';
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
    <aside className='relative flex h-full flex-col items-center transition-all'>
      <div className='flex min-h-0 flex-1 flex-col items-center'>
        <Button
          as={NextLink}
          className='h-16 w-16 flex-shrink-0'
          href='/dashboard'
          isIconOnly
          radius='none'
          variant={!team ? 'solid' : 'light'}
        >
          <Avatar src='https://i.pravatar.cc?img=1' />
        </Button>

        <ScrollShadow
          className='flex min-h-0 flex-1 flex-col overflow-y-auto'
          hideScrollBar
        >
          {teams?.map((t) => (
            <Button
              as={NextLink}
              className='h-16 w-16 flex-shrink-0'
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
        </ScrollShadow>

        <CreateTeamButton
          className='h-16 w-16 flex-shrink-0'
          radius='none'
          variant='light'
        />
      </div>

      <div className='flex flex-col items-center'>
        <div className='flex h-16 w-16 items-center justify-center'>
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
        </div>
        <div className='flex h-16 w-16 items-center justify-center'>
          <Button
            isIconOnly
            onPress={logout}
            startContent={<IconLogout />}
            variant='light'
          />
        </div>
      </div>
    </aside>
  );
}
