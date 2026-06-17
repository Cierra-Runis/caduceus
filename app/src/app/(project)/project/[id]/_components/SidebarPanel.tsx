'use client';

import { FileIcon } from 'lucide-react';
import { RefObject } from 'react';
import { ImperativePanelHandle, Panel } from 'react-resizable-panels';

import { cn } from '@/lib/utils';

export interface SidebarPanelProps {
  /// Path of the compile entry file, marked in the list. Null if none.
  entry: null | string;
  /// Path of the file currently open in the editor.
  focus: string;
  onSelect: (path: string) => void;
  paths: string[];
  sidebarPanelRef: RefObject<ImperativePanelHandle | null>;
}

export function SidebarPanel({
  entry,
  focus,
  onSelect,
  paths,
  sidebarPanelRef,
}: SidebarPanelProps) {
  return (
    <Panel
      collapsible
      defaultSize={0}
      id='sidebar'
      minSize={10}
      order={0}
      ref={sidebarPanelRef}
    >
      <ul className='flex flex-col py-2'>
        {paths.map((path) => (
          <li key={path}>
            <button
              className={cn(
                `flex w-full items-center gap-2 px-3 py-1 text-left text-sm`,
                path === focus ? 'bg-accent' : 'hover:bg-accent/50',
              )}
              onClick={() => onSelect(path)}
            >
              <FileIcon className='size-4 shrink-0 opacity-60' />
              <span className='truncate'>{path}</span>
              {path === entry && (
                <span className='ml-auto text-xs opacity-50'>entry</span>
              )}
            </button>
          </li>
        ))}
      </ul>
    </Panel>
  );
}
