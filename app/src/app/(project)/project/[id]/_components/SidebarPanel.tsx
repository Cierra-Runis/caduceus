'use client';

import { FileIcon, PencilIcon, PlusIcon, Trash2Icon } from 'lucide-react';
import { KeyboardEvent, RefObject, useState } from 'react';
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
  /// Create a root-level file with this name; returns whether it succeeded (a
  /// rejected name keeps the input open).
  onCreateFile: (name: string) => boolean;
  /// Delete the file with this id.
  onDelete: (id: string) => void;
  /// Rename the node with this id; returns whether it succeeded.
  onRename: (id: string, name: string) => boolean;
  onSelect: (id: string) => void;
  sidebarPanelRef: RefObject<null | PanelImperativeHandle>;
}

export function SidebarPanel({
  entry,
  files,
  focus,
  onCreateFile,
  onDelete,
  onRename,
  onSelect,
  sidebarPanelRef,
}: SidebarPanelProps) {
  // `creating` toggles the new-file input; `renamingId` marks which row is being
  // renamed. Only one of the two is ever active.
  const [creating, setCreating] = useState(false);
  const [renamingId, setRenamingId] = useState<null | string>(null);

  return (
    <Panel
      collapsible
      defaultSize={0}
      id='sidebar'
      minSize={10}
      panelRef={sidebarPanelRef}
    >
      <div className='flex items-center justify-between px-3 py-2'>
        <span className='text-xs font-medium opacity-60'>Files</span>
        <button
          aria-label='New file'
          className='rounded-sm p-1 hover:bg-accent'
          onClick={() => {
            setRenamingId(null);
            setCreating(true);
          }}
          title='New file'
        >
          <PlusIcon className='size-4' />
        </button>
      </div>

      <ul className='flex flex-col pb-2'>
        {creating && (
          <li className='px-3 py-1'>
            <NameInput
              onCancel={() => setCreating(false)}
              onSubmit={(name) => {
                if (onCreateFile(name)) setCreating(false);
              }}
              placeholder='file name'
            />
          </li>
        )}
        {files.map(({ id, path }) =>
          renamingId === id ? (
            <li className='px-3 py-1' key={id}>
              <NameInput
                initial={basename(path)}
                onCancel={() => setRenamingId(null)}
                onSubmit={(name) => {
                  if (name === basename(path) || onRename(id, name)) {
                    setRenamingId(null);
                  }
                }}
              />
            </li>
          ) : (
            <li className='group flex items-center' key={id}>
              <button
                aria-current={id === focus ? 'true' : undefined}
                className={cn(
                  `flex min-w-0 flex-1 items-center gap-2 px-3 py-1 text-left
                  text-sm`,
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
              <span
                className={`
                  flex shrink-0 items-center pr-2 opacity-0
                  group-focus-within:opacity-100 group-hover:opacity-100
                `}
              >
                <button
                  aria-label={`Rename ${path}`}
                  className='rounded-sm p-1 hover:bg-accent'
                  onClick={() => {
                    setCreating(false);
                    setRenamingId(id);
                  }}
                  title='Rename'
                >
                  <PencilIcon className='size-3.5' />
                </button>
                <button
                  aria-label={`Delete ${path}`}
                  className='rounded-sm p-1 hover:bg-accent'
                  onClick={() => onDelete(id)}
                  title='Delete'
                >
                  <Trash2Icon className='size-3.5' />
                </button>
              </span>
            </li>
          ),
        )}
      </ul>
    </Panel>
  );
}

/// The last segment of a `/`-path — the node's own name, which is what a rename
/// edits.
function basename(path: string): string {
  const i = path.lastIndexOf('/');
  return i === -1 ? path : path.slice(i + 1);
}

/// A small inline text input for creating/renaming: submits its trimmed value on
/// Enter, cancels on Escape or blur.
function NameInput({
  initial = '',
  onCancel,
  onSubmit,
  placeholder,
}: {
  initial?: string;
  onCancel: () => void;
  onSubmit: (value: string) => void;
  placeholder?: string;
}) {
  const [value, setValue] = useState(initial);
  const onKeyDown = (event: KeyboardEvent<HTMLInputElement>) => {
    if (event.key === 'Enter') onSubmit(value.trim());
    else if (event.key === 'Escape') onCancel();
  };
  return (
    <input
      autoFocus
      className={`
        w-full rounded-sm border bg-background px-2 py-1 text-sm outline-none
        focus:border-ring
      `}
      onBlur={onCancel}
      onChange={(event) => setValue(event.target.value)}
      onKeyDown={onKeyDown}
      placeholder={placeholder}
      value={value}
    />
  );
}
