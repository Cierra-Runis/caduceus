'use client';

import {
    FilesIcon,
    HomeIcon,
    SearchIcon,
    SettingsIcon,
} from 'lucide-react';
import NextLink from 'next/link';

import { Button } from '@/components/ui/button';
import { ScrollArea } from '@/components/ui/scroll-area';
import { cn } from '@/lib/utils';

export interface SidebarProps {
  /// The open view, or null when the panel is collapsed (nothing highlighted).
  activeView: null | SidebarView;
  onSelectView: (view: SidebarView) => void;
}

export type SidebarView = 'files' | 'search';

// VS Code-style activity bar. Each view button toggles / switches the sidebar
// panel; the active one is highlighted (a filled variant plus a left accent
// bar, like VS Code).
export function Sidebar({ activeView, onSelectView }: SidebarProps) {
  return (
    <ScrollArea
      className={`
        relative flex h-full min-w-12 flex-col items-center overflow-auto pt-11
        transition-all
      `}
    >
      <ScrollArea className='flex w-full flex-1 flex-col'>
        <ActivityButton
          active={activeView === 'files'}
          label='Files'
          onClick={() => onSelectView('files')}
        >
          <FilesIcon />
        </ActivityButton>
        <ActivityButton
          active={activeView === 'search'}
          label='Search'
          onClick={() => onSelectView('search')}
        >
          <SearchIcon />
        </ActivityButton>
      </ScrollArea>
      <div className='flex w-full shrink-0 flex-col items-center'>
        <div
          className={`
            flex aspect-square h-auto w-full shrink-0 items-center
            justify-center
          `}
        >
          <Button disabled size='icon' variant='ghost'>
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

function ActivityButton({
  active,
  children,
  label,
  onClick,
}: {
  active: boolean;
  children: React.ReactNode;
  label: string;
  onClick: () => void;
}) {
  return (
    <div className='relative'>
      {active && (
        <span className='absolute inset-y-1 left-0 w-0.5 rounded-full bg-primary' />
      )}
      <Button
        aria-label={label}
        aria-pressed={active}
        className='aspect-square h-auto w-full shrink-0'
        onClick={onClick}
        size='icon'
        title={label}
        variant={active ? 'secondary' : 'ghost'}
      >
        <span className={cn(active ? 'opacity-100' : 'opacity-70')}>
          {children}
        </span>
      </Button>
    </div>
  );
}
