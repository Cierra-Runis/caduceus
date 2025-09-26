'use client';

import { Button } from '@heroui/button';
import { ScrollShadow } from '@heroui/scroll-shadow';
import {
  IconArchive,
  IconLogout,
  IconMap,
  IconPencil,
  IconSearch,
  IconSettings,
} from '@tabler/icons-react';
import { useCallback } from 'react';

import { logout } from '@/actions/auth';

import { SidebarPanelProps } from './SidebarPanel';

export function Sidebar({ sidebarPanelRef }: SidebarPanelProps) {
  const toggleSidebarPanel = useCallback(() => {
    const current = sidebarPanelRef?.current;
    if (!current) return;
    return current.isCollapsed() ? current.expand() : current.collapse();
  }, [sidebarPanelRef]);

  return (
    <ScrollShadow
      className='bg-default-50 relative flex h-full min-w-18 flex-col items-center overflow-auto pt-11 transition-all'
      hideScrollBar
    >
      <ScrollShadow className='flex w-full flex-1 flex-col' hideScrollBar>
        <Button
          className='aspect-square h-auto w-full flex-shrink-0'
          isIconOnly
          onPress={toggleSidebarPanel}
          radius='none'
          variant='light'
        >
          <IconArchive />
        </Button>
        <Button
          className='aspect-square h-auto w-full flex-shrink-0'
          isIconOnly
          onPress={toggleSidebarPanel}
          radius='none'
          variant='light'
        >
          <IconSearch />
        </Button>
        <Button
          className='aspect-square h-auto w-full flex-shrink-0'
          isIconOnly
          onPress={toggleSidebarPanel}
          radius='none'
          variant='light'
        >
          <IconMap />
        </Button>
        <Button
          className='aspect-square h-auto w-full flex-shrink-0'
          isIconOnly
          onPress={toggleSidebarPanel}
          radius='none'
          variant='light'
        >
          <IconPencil />
        </Button>
      </ScrollShadow>
      <div className='flex w-full flex-shrink-0 flex-col items-center'>
        <div className='flex aspect-square h-auto w-full flex-shrink-0 items-center justify-center'>
          <Button
            isIconOnly
            onPress={toggleSidebarPanel}
            startContent={<IconSettings />}
            variant='light'
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
