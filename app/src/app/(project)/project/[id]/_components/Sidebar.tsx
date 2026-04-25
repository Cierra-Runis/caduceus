'use client';

import {
  IconArchive,
  IconHome,
  IconMap,
  IconPencil,
  IconSearch,
  IconSettings,
} from '@tabler/icons-react';
import NextLink from 'next/link';
import { useCallback } from 'react';

import { Button } from '@/components/ui/button';
import { ScrollArea } from '@/components/ui/scroll-area';

import { SidebarPanelProps } from './SidebarPanel';

export function Sidebar({ sidebarPanelRef }: SidebarPanelProps) {
  const toggleSidebarPanel = useCallback(() => {
    const current = sidebarPanelRef?.current;
    if (!current) return;
    return current.isCollapsed() ? current.expand() : current.collapse();
  }, [sidebarPanelRef]);

  return (
    <ScrollArea
      className={`
        relative flex h-full min-w-18 flex-col items-center overflow-auto pt-11
        transition-all
      `}
    >
      <ScrollArea className='flex w-full flex-1 flex-col'>
        <Button
          className='aspect-square h-auto w-full shrink-0'
          onClick={toggleSidebarPanel}
          size='icon'
          variant='ghost'
        >
          <IconArchive />
        </Button>
        <Button
          className='aspect-square h-auto w-full shrink-0'
          onClick={toggleSidebarPanel}
          size='icon'
          variant='ghost'
        >
          <IconSearch />
        </Button>
        <Button
          className='aspect-square h-auto w-full shrink-0'
          onClick={toggleSidebarPanel}
          size='icon'
          variant='ghost'
        >
          <IconMap />
        </Button>
        <Button
          className='aspect-square h-auto w-full shrink-0'
          onClick={toggleSidebarPanel}
          size='icon'
          variant='ghost'
        >
          <IconPencil />
        </Button>
      </ScrollArea>
      <div className='flex w-full shrink-0 flex-col items-center'>
        <div
          className={`
            flex aspect-square h-auto w-full shrink-0 items-center
            justify-center
          `}
        >
          <Button onClick={toggleSidebarPanel} size='icon' variant='ghost'>
            <IconSettings />
          </Button>
        </div>
        <div
          className={`
            flex aspect-square h-auto w-full shrink-0 items-center
            justify-center
          `}
        >
          <Button asChild size='icon' variant='ghost'>
            <NextLink href='/'>
              <IconHome />
            </NextLink>
          </Button>
        </div>
      </div>
    </ScrollArea>
  );
}
