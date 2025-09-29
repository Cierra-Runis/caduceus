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
import { UserMeTooltip } from './tooltips/UserMeTooltip';

export function Sidebar() {
  const { team } = useParams();
  const pathname = usePathname();

  const isInSettings =
    pathname.endsWith('/settings') || pathname.endsWith('/manage');

  return (
    <ScrollShadow
      className='bg-content1 relative flex h-full min-w-18 flex-col items-center overflow-auto pt-11 transition-all'
      hideScrollBar
    >
      <Button
        as={NextLink}
        className='aspect-square h-auto w-full flex-shrink-0'
        href='/'
        isIconOnly
        radius='none'
        variant={!team ? 'solid' : 'light'}
      >
        <UserMeTooltip />
      </Button>
      <TeamList />
      <div className='flex w-full flex-shrink-0 flex-col items-center'>
        <div className='flex aspect-square h-auto w-full flex-shrink-0 items-center justify-center'>
          <Button
            as={NextLink}
            href={
              isInSettings
                ? team
                  ? `/dashboard/team/${team}`
                  : '/'
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

function TeamList() {
  const { team } = useParams();
  const { data } = useUserTeams();

  return (
    <ScrollShadow className='flex w-full flex-1 flex-col' hideScrollBar>
      {data?.payload?.map((t) => (
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
            <Avatar radius='sm' src={t.avatar_uri ?? '/icon.svg'} />
          </Tooltip>
        </Button>
      ))}
      <CreateTeamButton
        className='aspect-square h-auto w-full flex-shrink-0'
        radius='none'
        variant='light'
      />
    </ScrollShadow>
  );
}
