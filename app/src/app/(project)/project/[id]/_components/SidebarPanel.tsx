'use client';

import { FileIcon } from 'lucide-react';
import { RefObject } from 'react';
import { Panel, PanelImperativeHandle } from 'react-resizable-panels';

import { cn } from '@/lib/utils';

export interface SidebarPanelProps {
  /// Id of the compile entry file, marked in the list. Null if none.
  entry: null | string;
  /// The text files to list, as `{ id, path }` — selected/keyed by id, shown
  /// by path.
  files: { id: string; path: string }[];
  /// Id of the file currently open in the editor.
  focus: string;
  onSelect: (id: string) => void;
  sidebarPanelRef: RefObject<null | PanelImperativeHandle>;
}

export function SidebarPanel({
  entry,
  files,
  focus,
  onSelect,
  sidebarPanelRef,
}: SidebarPanelProps) {
  return (
    <Panel
      collapsible
      defaultSize={0}
      id='sidebar'
      minSize={10}
      panelRef={sidebarPanelRef}
    >
      <ul className='flex flex-col py-2'>
        {files.map(({ id, path }) => (
          <li key={id}>
            <button
              aria-current={id === focus ? 'true' : undefined}
              className={cn(
                `flex w-full items-center gap-2 px-3 py-1 text-left text-sm`,
                id === focus ? 'bg-accent' : 'hover:bg-accent/50',
              )}
              onClick={() => onSelect(id)}
            >
              <FileIcon className='size-4 shrink-0 opacity-60' />
              <span className='truncate'>{path}</span>
              {id === entry && (
                <span className='ml-auto text-xs opacity-50'>entry</span>
              )}
            </button>
          </li>
        ))}
      </ul>
    </Panel>
  );
}
