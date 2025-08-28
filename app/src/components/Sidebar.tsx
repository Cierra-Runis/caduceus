'use client';

import { Avatar } from '@heroui/avatar';
import { Button } from '@heroui/button';
import { Tab, Tabs } from '@heroui/tabs';
import { IconLogout, IconSettings } from '@tabler/icons-react';
import NextLink from 'next/link';
import { useParams, usePathname } from 'next/navigation';

import { logout } from '@/actions/auth';

export default function Sidebar() {
  const { team } = useParams();
  const pathname = usePathname();

  const isInSettings =
    pathname.endsWith('/settings') || pathname.endsWith('/manage');

  return (
    <aside className='relative flex flex-col items-center justify-between py-3 transition-all'>
      <div className='flex flex-col items-center'>
        <Tabs
          classNames={{
            tab: 'w-auto h-auto p-2',
            tabList: 'space-y-4 bg-transparent',
          }}
          isVertical
          selectedKey={team ? `/dashboard/team/${team}` : '/dashboard'}
        >
          <Tab
            as={NextLink}
            href='/dashboard'
            key='/dashboard'
            title={<Avatar src='https://i.pravatar.cc?img=1' />}
          />

          <Tab
            as={NextLink}
            href='/dashboard/team/grjfkesghzishfkjhwf'
            key='/dashboard/team/grjfkesghzishfkjhwf'
            title={<Avatar src='https://i.pravatar.cc?img=2' />}
          />
          <Tab
            as={NextLink}
            href='/dashboard/team/esglihURGbwkfhaqwih'
            key='/dashboard/team/esglihURGbwkfhaqwih'
            title={<Avatar src='https://i.pravatar.cc?img=3' />}
          />
        </Tabs>
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
