'use client';

import { FileIcon, ImageIcon, UploadIcon } from 'lucide-react';
import { useTranslations } from 'next-intl';
import { RefObject, useRef } from 'react';
import { Panel, PanelImperativeHandle } from 'react-resizable-panels';

import { Spinner } from '@/components/ui/spinner';
import { cn } from '@/lib/utils';

/// A single entry in the project file tree.
export interface SidebarFile {
  kind: 'binary' | 'text';
  path: string;
}

export interface SidebarPanelProps {
  /// Path of the compile entry file, marked in the list. Null if none.
  entry: null | string;
  /// The whole project file tree, text and binary alike.
  files: SidebarFile[];
  /// Path of the file currently open in the editor.
  focus: string;
  onSelect: (path: string) => void;
  /// Upload a picked file as a project asset.
  onUpload: (file: File) => void;
  sidebarPanelRef: RefObject<null | PanelImperativeHandle>;
  /// An upload is in flight; the control is disabled and shows a spinner.
  uploading: boolean;
}

export function SidebarPanel({
  entry,
  files,
  focus,
  onSelect,
  onUpload,
  sidebarPanelRef,
  uploading,
}: SidebarPanelProps) {
  const t = useTranslations('Project');
  const inputRef = useRef<HTMLInputElement>(null);

  return (
    <Panel
      collapsible
      defaultSize={0}
      id='sidebar'
      minSize={10}
      panelRef={sidebarPanelRef}
    >
      <div className='flex items-center justify-between px-3 py-2'>
        <span className='text-xs font-medium opacity-60'>{t('files')}</span>
        <button
          className={cn(
            `flex items-center gap-1 rounded-sm px-2 py-1 text-xs`,
            uploading ? 'opacity-50' : 'hover:bg-accent',
          )}
          disabled={uploading}
          onClick={() => inputRef.current?.click()}
          type='button'
        >
          {uploading ? (
            <Spinner className='size-3' />
          ) : (
            <UploadIcon className='size-3' />
          )}
          {t('asset.upload')}
        </button>
        <input
          className='hidden'
          onChange={(event) => {
            const file = event.target.files?.[0];
            // Reset so re-picking the same file fires `onChange` again.
            event.target.value = '';
            if (file) onUpload(file);
          }}
          ref={inputRef}
          type='file'
        />
      </div>
      <ul className='flex flex-col pb-2'>
        {files.map((file) => {
          const isBinary = file.kind === 'binary';
          return (
            <li key={file.path}>
              <button
                className={cn(
                  `flex w-full items-center gap-2 px-3 py-1 text-left text-sm`,
                  file.path === focus ? 'bg-accent' : 'hover:bg-accent/50',
                  isBinary && 'cursor-default opacity-70 hover:bg-transparent',
                )}
                // Binary assets have no text buffer to open; the row is a
                // reference for `#image(...)`, not an editor target.
                disabled={isBinary}
                onClick={() => onSelect(file.path)}
                type='button'
              >
                {isBinary ? (
                  <ImageIcon className='size-4 shrink-0 opacity-60' />
                ) : (
                  <FileIcon className='size-4 shrink-0 opacity-60' />
                )}
                <span className='truncate'>{file.path}</span>
                {file.path === entry && (
                  <span className='ml-auto text-xs opacity-50'>entry</span>
                )}
              </button>
            </li>
          );
        })}
      </ul>
    </Panel>
  );
}
