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
    <ScrollShadow
      className='relative flex h-full w-16 flex-col items-center overflow-auto transition-all'
      hideScrollBar
    >
      <Button
        as={NextLink}
        className='aspect-square h-auto w-full flex-shrink-0'
        href='/dashboard'
        isIconOnly
        radius='none'
        variant={!team ? 'solid' : 'light'}
      >
        <Avatar src='https://i.pravatar.cc?img=1' />
      </Button>

      <ScrollShadow className='flex w-full flex-1 flex-col' hideScrollBar>
        {teams?.map((t) => (
          <Button
            as={NextLink}
            className='aspect-square h-auto w-full flex-shrink-0'
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
        <CreateTeamButton
          className='aspect-square h-auto w-full flex-shrink-0'
          radius='none'
          variant='light'
        />
      </ScrollShadow>

      <div className='flex w-full flex-shrink-0 flex-col items-center'>
        <div className='flex aspect-square h-auto w-full flex-shrink-0 items-center justify-center'>
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
        <div className='flex aspect-square h-auto w-full flex-shrink-0 items-center justify-center'>
          <Button
            isIconOnly
            onPress={logout}
            startContent={<IconLogout />}
            variant='light'
          />
        </div>
      </div>
    </ScrollShadow>
  );
}
