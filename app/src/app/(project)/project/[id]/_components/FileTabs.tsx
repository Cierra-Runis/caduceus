'use client';

import { XIcon } from 'lucide-react';
import { useTranslations } from 'next-intl';

import { cn } from '@/lib/utils';

/// One open editor tab: a file id, its display name, and whether its live text
/// has drifted from its saved blob (the unsaved-changes dot).
export interface FileTab {
  dirty: boolean;
  id: string;
  name: string;
}

export interface FileTabsProps {
  /// Id of the tab currently shown in the editor.
  activeId: string;
  /// Close the tab with this id.
  onClose: (id: string) => void;
  /// Focus the tab with this id.
  onSelect: (id: string) => void;
  /// Open tabs, in order.
  tabs: FileTab[];
}

/// A VS Code-style tab strip above the editor. Each tab shows the file name and,
/// when it has unsaved changes, a dot that turns into a close button on hover
/// (an always-visible close button on hover otherwise). Middle-click also closes.
export function FileTabs({ activeId, onClose, onSelect, tabs }: FileTabsProps) {
  const t = useTranslations('Editor');
  if (tabs.length === 0) return null;

  return (
    <div className='flex shrink-0 overflow-x-auto border-b text-sm'>
      {tabs.map((tab) => {
        const active = tab.id === activeId;
        return (
          <div
            className={cn(
              'group flex items-center border-r',
              active ? 'bg-background' : 'bg-muted/40 hover:bg-muted/70',
            )}
            key={tab.id}
            onAuxClick={(event) => {
              if (event.button === 1) {
                event.preventDefault();
                onClose(tab.id);
              }
            }}
          >
            <button
              className={cn(
                'max-w-40 truncate py-1.5 pr-1 pl-3',
                active ? 'opacity-100' : 'opacity-70',
              )}
              onClick={() => onSelect(tab.id)}
              title={tab.name}
            >
              {tab.name}
            </button>
            <button
              aria-label={t('closeTab', { name: tab.name })}
              className='relative mr-1 grid size-5 place-items-center rounded-sm hover:bg-accent'
              onClick={() => onClose(tab.id)}
            >
              {/* Dirty dot: shown when dirty and not hovering; the × takes over
                  on hover. A clean tab shows the × only on hover. */}
              {tab.dirty && (
                <span className='size-2 rounded-full bg-current opacity-70 group-hover:hidden' />
              )}
              <XIcon
                className={cn(
                  'size-3.5',
                  tab.dirty ? 'hidden group-hover:block' : 'opacity-0 group-hover:opacity-100',
                )}
              />
            </button>
          </div>
        );
      })}
    </div>
  );
}
