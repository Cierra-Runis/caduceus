'use client';

import { SettingsIcon } from 'lucide-react';
import NextLink from 'next/link';
import { useParams, usePathname } from 'next/navigation';

import { CreateTeamButton } from '@/components/buttons/CreateTeamButton';
import { Avatar, AvatarImage } from '@/components/ui/avatar';
import { Button } from '@/components/ui/button';
import { ScrollArea } from '@/components/ui/scroll-area';
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarHeader,
} from '@/components/ui/sidebar';
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from '@/components/ui/tooltip';
import { useUserMe } from '@/hooks/api/user/me';
import { useUserTeams } from '@/hooks/api/user/team';

type Params = Awaited<PageProps<'/dashboard/team/[id]'>['params']>;

export function SideBar() {
  const { id } = useParams<Params>();
  const pathname = usePathname();

  const isInSettings = pathname.endsWith('/settings');

  return (
    <Sidebar className='pt-11'>
      <SidebarHeader className='p-0'>
        <UserMeTooltip id={id} />
      </SidebarHeader>
      <SidebarContent>
        <TeamList />
      </SidebarContent>
      <SidebarFooter>
        <Button
          asChild
          size='icon'
          variant={isInSettings ? 'secondary' : 'ghost'}
        >
          <NextLink
            href={
              isInSettings
                ? id
                  ? `/dashboard/team/${id}`
                  : '/'
                : id
                  ? `/dashboard/team/${id}/settings`
                  : '/dashboard/settings'
            }
          >
            <SettingsIcon />
          </NextLink>
        </Button>
      </SidebarFooter>
    </Sidebar>
  );
}

export function UserMeTooltip({ id }: { id?: string }) {
  const { data, isLoading } = useUserMe();

  return (
    <Tooltip>
      <TooltipTrigger
        className='aspect-square h-auto w-full'
        disabled={isLoading || !data}
      >
        <Button
          asChild
          className='aspect-square h-auto w-full rounded-none border-none'
          variant={id ? 'ghost' : 'outline'}
        >
          <NextLink href='/'>
            <Avatar size='lg'>
              <AvatarImage src={data?.payload.avatar_uri ?? '/icon.svg'} />
            </Avatar>
          </NextLink>
        </Button>
      </TooltipTrigger>
      <TooltipContent side='right'>{data?.payload?.username}</TooltipContent>
    </Tooltip>
  );
}

function TeamList() {
  const { id } = useParams<Params>();
  const { data } = useUserTeams();

  return (
    <ScrollArea className='flex w-full flex-1 flex-col'>
      {data?.payload?.map((t) => (
        <Tooltip key={t.id}>
          <TooltipTrigger className='aspect-square h-auto w-full'>
            <Button
              asChild
              className='aspect-square h-auto w-full rounded-none border-none'
              variant={id === t.id ? 'outline' : 'ghost'}
            >
              <NextLink href={`/dashboard/team/${t.id}`}>
                <Avatar size='lg'>
                  <AvatarImage src={t.avatar_uri ?? '/icon.svg'} />
                </Avatar>
              </NextLink>
            </Button>
          </TooltipTrigger>
          <TooltipContent side='right'>{t.name}</TooltipContent>
        </Tooltip>
      ))}
      <CreateTeamButton />
    </ScrollArea>
  );
}
