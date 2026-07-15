'use client';

import {
    ArchiveIcon,
    EditIcon,
    HomeIcon,
    MapIcon,
    SearchIcon,
    SettingsIcon,
} from 'lucide-react';
import NextLink from 'next/link';
import { RefObject, useCallback } from 'react';
import { PanelImperativeHandle } from 'react-resizable-panels';

import { Button } from '@/components/ui/button';
import { ScrollArea } from '@/components/ui/scroll-area';

export interface SidebarProps {
  sidebarPanelRef: RefObject<null | PanelImperativeHandle>;
}

export function Sidebar({ sidebarPanelRef }: SidebarProps) {
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
          <ArchiveIcon />
        </Button>
        <Button
          className='aspect-square h-auto w-full shrink-0'
          onClick={toggleSidebarPanel}
          size='icon'
          variant='ghost'
        >
          <SearchIcon />
        </Button>
        <Button
          className='aspect-square h-auto w-full shrink-0'
          onClick={toggleSidebarPanel}
          size='icon'
          variant='ghost'
        >
          <MapIcon />
        </Button>
        <Button
          className='aspect-square h-auto w-full shrink-0'
          onClick={toggleSidebarPanel}
          size='icon'
          variant='ghost'
        >
          <EditIcon />
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
            <SettingsIcon />
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
              <HomeIcon />
            </NextLink>
          </Button>
        </div>
      </div>
    </ScrollArea>
  );
}
