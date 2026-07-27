'use client';

import { FilesIcon, HomeIcon, SettingsIcon } from 'lucide-react';
import { useTranslations } from 'next-intl';
import NextLink from 'next/link';
import { RefObject, useCallback } from 'react';
import { PanelImperativeHandle } from 'react-resizable-panels';

import { Button } from '@/components/ui/button';
import { cn } from '@/lib/utils';

export interface SidebarProps {
  /// The view currently shown in the sidebar panel.
  activePanel: SidebarView;
  /// Switch the sidebar panel to this view.
  onSelectPanel: (view: SidebarView) => void;
  sidebarPanelRef: RefObject<null | PanelImperativeHandle>;
}

/// Which view the sidebar panel is showing.
export type SidebarView = 'files' | 'settings';

export function Sidebar({
  activePanel,
  onSelectPanel,
  sidebarPanelRef,
}: SidebarProps) {
  const t = useTranslations('Editor');

  // Clicking an activity-bar icon shows its view and expands the panel; clicking
  // the already-active view collapses the panel (VS Code's activity bar).
  const selectPanel = useCallback(
    (view: SidebarView) => {
      const current = sidebarPanelRef?.current;
      if (view === activePanel && current && !current.isCollapsed()) {
        current.collapse();
        return;
      }
      onSelectPanel(view);
      current?.expand();
    },
    [activePanel, onSelectPanel, sidebarPanelRef],
  );

  const items: { icon: typeof FilesIcon; label: string; view: SidebarView }[] = [
    { icon: FilesIcon, label: t('files'), view: 'files' },
    { icon: SettingsIcon, label: t('settings'), view: 'settings' },
  ];

  return (
    <div
      className={`
        relative flex h-full min-w-14 flex-col items-center overflow-auto pt-11
      `}
    >
      <div className='flex w-full flex-1 flex-col'>
        {items.map(({ icon: Icon, label, view }) => (
          <Button
            aria-label={label}
            aria-pressed={view === activePanel}
            className={cn(
              'aspect-square h-auto w-full shrink-0',
              view === activePanel && 'bg-accent',
            )}
            key={view}
            onClick={() => selectPanel(view)}
            size='icon'
            title={label}
            variant='ghost'
          >
            <Icon />
          </Button>
        ))}
      </div>
      <div className='flex w-full shrink-0 flex-col items-center'>
        <Button asChild size='icon' variant='ghost'>
          <NextLink href='/'>
            <HomeIcon />
          </NextLink>
        </Button>
      </div>
    </div>
  );
}
